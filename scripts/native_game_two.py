"""Replay the reviewed Expert continuation through both manager interfaces."""

import argparse
from contextlib import ExitStack, contextmanager
from dataclasses import asdict, dataclass
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

from PIL import ImageChops

from native_buy_round import (
    build, capture_when, check_display, click, close, launch, position, require, wait_for, xdotool,
)


FIXTURE = Path("tests/fixtures/expert-game-two-start.json")
SIZES = (1080, 720), (3840, 2160)


@dataclass(frozen=True)
class Click:
    seat: str
    kind: str
    name: str
    label: str
    point: tuple[int, int]
    save_digest: str


def file_digest(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        while chunk := stream.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


@contextmanager
def manager(executable, save, assets, evidence, transcript, seat, *, args=()):
    command = [executable, "--save", str(save), "--agent-seat", seat, *args]
    if assets:
        command += ["--assets", str(assets)]
    env = os.environ.copy()
    env["RUST_LOG"] = "info"
    log = evidence / f"{seat}-agent.log"
    with log.open("w") as stderr, subprocess.Popen(
        command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
        text=True, env=env,
    ) as process:
        def receive(request, status):
            line = process.stdout.readline()
            require(line, f"{seat} manager seat stopped responding. See {log}")
            result = json.loads(line)
            record = {"seat": seat, "request": request, "response": result}
            transcript.write(json.dumps(record) + "\n")
            transcript.flush()
            require(
                result["ok"] and result["status"] == status,
                f"{seat} manager seat expected {status}, received {result['status']}. "
                f"{result.get('error', '')} See {log} and {transcript.name}",
            )
            return result

        def request(operation, status, **arguments):
            packet = {"op": operation, **arguments}
            process.stdin.write(json.dumps(packet) + "\n")
            process.stdin.flush()
            return receive(packet, status)

        try:
            receive(None, "ready")
            yield request
            request("quit", "bye")
            require(process.wait(timeout=5) == 0, f"{seat} manager seat failed at exit. See {log}")
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=4)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()


def observe(request):
    reply = request("observe", "observed")
    if not reply["settled"]:
        reply = request("settle", "settled")
    require(
        reply["settled"] and not reply["table"]["refresh_error"],
        "Could not observe the settled, current position",
    )
    return reply


def activate(request, seat, trace, save, kind, name):
    before = observe(request)
    control = next((item for item in before["table"]["controls"] if item[kind] == name), None)
    require(control is not None and control["on_screen"], f"{seat} cannot reach {kind} {name}")
    point = tuple(round(axis) for axis in control["position"])
    after = request("activate", "activated", revision=before["revision"], **{kind: name})
    trace.append(Click(seat, kind, name, control["label"], point, file_digest(save)))
    return after["observation"]


def purchase(choose, identity, demotion, replacement):
    market = choose("component", "inspect-slot-5")
    card = market["offers"][4]["player"]["card"]
    require(card["identity"] == identity, f"Expected {identity} in slot five")
    signed = choose("choice", f"sign-5-{card['instance']}")
    require(
        signed["events"][-1]["card"] == card
        and signed["offers"][4]["player"]["card"]["identity"] == replacement,
        f"Expected to sign {identity} and reveal {replacement}",
    )
    eligible = signed["decision"]["eligible"]
    demoted = next((card for card in eligible if card["identity"] == demotion), None)
    require(demoted is not None, f"{demotion} is missing from the eligible demotions")
    choose("component", f"inspect-{demotion}")
    choose("choice", f"demote-{demotion}-{demoted['instance']}")
    return choose("choice", "pass")


def play(choose, identity):
    choose("component", f"inspect-{identity}")
    return choose("choice", f"play-{identity}")


def replay_managers(executable, assets, evidence, save, trace):
    with ExitStack() as exits:
        transcript = exits.enter_context((evidence / "manager-transcript.jsonl").open("w"))
        sf = exits.enter_context(manager(executable, save, assets, evidence, transcript, "sf"))
        ny = exits.enter_context(manager(executable, save, assets, evidence, transcript, "ny"))

        def choose_sf(kind, name):
            return activate(sf, "sf", trace, save, kind, name)

        def choose_ny(kind, name):
            return activate(ny, "ny", trace, save, kind, name)

        before = save.read_bytes()
        activate(ny, "ny", [], save, "component", "inspect-slot-2")
        require(any(control["component"] == "close-inspection" for control in observe(ny)["table"]["controls"]),
                "NY must be able to inspect a Free Agent while waiting for the order")
        require(save.read_bytes() == before, "Revenue offer inspection changed the save")
        choose_sf("choice", "buy-first-sf")
        choose_sf("component", "inspect-slot-2")
        purchase(choose_sf, "FA-065", "ST-004", "FA-067")
        closed = purchase(choose_ny, "FA-067", "ST-056", "FA-068")
        require(
            closed["balances"] == [0, 0] and [club["active"] for club in closed["rosters"]] == [15, 15],
            "The Buy Round must end with 15 active players per team and zero balance",
        )
        clubs = position(save)["clubs"]
        before = save.read_bytes()
        for team in ("sf", "ny"):
            choose_sf("component", f"open-dugout-{team}")
            choose_sf("component", "close-dugout")
        require(save.read_bytes() == before, "Opening the public Dugouts changed the save")
        activate(ny, "ny", [], save, "component", "open-dugout-sf")
        require(observe(ny)["table"]["dugout_inspection"] is not None,
                "NY must be able to inspect SF's public Dugout before Game 2")
        require(save.read_bytes() == before, "The waiting manager's Dugout inspection changed the save")
        sf_drawn = choose_sf("choice", "start-game-two")
        shutil.copyfile(save, evidence / "sf-hand-position.json")
        ny_drawn = observe(ny)["observation"]
        require(
            sf_drawn["standing"] == ny_drawn["standing"] == {"game": 2, "home": "new_york", "wins": [0, 1]},
            "NY must host game two with its game-one win",
        )
        for index, view in enumerate((sf_drawn, ny_drawn)):
            own = view["rosters"][index]
            other = view["rosters"][1 - index]
            require(own["hand"] == clubs[index]["lineup"][-1:-7:-1], "Draw changed the Lineup order")
            require("hand" not in other, "The observing manager saw the other hand")

        play(choose_sf, "ST-013")
        shutil.copyfile(save, evidence / "ny-hand-position.json")
        play(choose_ny, "FA-067")
        pending = play(choose_sf, "FA-065")
        observer = observe(ny)["observation"]
        targets = pending["decision"]["targets"]
        require([target["cancelled"] for target in targets] == ["single", "home_run"],
                "Foxx must be able to cancel either of Maris's hits")
        require(observer["field"] == pending["field"] and observer["decision"] is None,
                "NY must see the pending field without an action")

        before = save.read_bytes()
        for box in (1, 2):
            preview = choose_sf("component", f"glove-hit-box-{box}")
            require(preview == pending and save.read_bytes() == before, "A Glove preview committed a change")

        committed = choose_sf("choice", "commit-glove-target")
        observed = [observe(request) for request in (sf, ny)]
        observer = observed[1]["observation"]
        field = committed["field"]
        for reply in observed:
            faces = {item["component"]: item["label"] for item in reply["table"]["controls"]}
            require(faces.get("inspect-glove-batter") == "Inspect Mickey Maris, NY In Play / Previous batter"
                    and faces.get("inspect-glove-defender") == "Inspect Frankie Foxx, SF In Play / Threatened hits",
                    "Both seats must identify Foxx as the current threat and Maris as the previous batter")
        require(committed["decision"] is None and observer["decision"] is None and field == observer["field"],
                "Both seats must see the same completed field without a pending action")
        sf_diamond, ny_diamond = field["diamonds"]
        maris = ny_drawn["rosters"][1]["hand"][0]
        foxx = sf_drawn["rosters"][0]["hand"][0]
        require(
            sf_diamond["score"] == ny_diamond["score"] == 0
            and ny_diamond["bases"] == [{"card": maris, "speed": "average"}, None, None],
            "Cancelling the home run must leave the acquired Maris on first at 0–0",
        )
        require(
            [(hit["hit"], hit["target"]["card"]) for hit in field["threatened"]] == [("double", foxx)],
            "Foxx's double must be the only pending threat",
        )


def replay_desktop(executable, assets, evidence, save, trace, size):
    from native_glove import choice_point, no_choices, pawn_at, selection_is
    from native_market import GOLD, PAPER, annotation, card_boxes, client_bounds, move
    from native_purchase_order import panel_visible, point
    from native_signing import center, demotion_action, finish_buttons, press, secondary_offer_faces, secondary_offers
    from native_stadium import action_span, check_park, faces_visible, geometry

    width, height = size
    scale = min(width / 1080, height / 720)
    stadium_scale, park, hand_faces = geometry(size)
    offer_faces = None
    selected_face = None
    demotion_market = None

    def matchup(image):
        return image.getpixel(point(size, 0.04, 0.26)) == (36, 60, 64)

    def dugouts(image):
        return not matchup(image) and all(
            image.getpixel(point(size, x, 0.45)) not in ((21, 40, 47), (36, 60, 64), (64, 94, 99))
            for x in (0.134, 0.317, 0.5, 0.683, 0.866)
        )

    def pending_demotion(image):
        return secondary_offers(image) and card_boxes(image)

    def open_window(exits, seat, name):
        x = width + 32 if size == SIZES[0] and seat == "ny" else 0
        game, window, _ = exits.enter_context(launch(
            executable, save, assets, evidence / f"{name}.log", seat=seat, origin=(x, 0),
        ))
        xdotool("windowsize", "--sync", window, *size)
        bounds = (x, 0, x + width, height)
        wait_for(lambda: client_bounds(window) == bounds, f"actual {width} × {height} {seat} client")
        (evidence / f"{name}-geometry.json").write_text(json.dumps({"bounds": bounds}) + "\n")
        return game, window, bounds

    def aim(entry, capture):
        nonlocal offer_faces, selected_face, demotion_market
        name = entry.name
        seat = entry.seat
        if name == "buy-first-sf":
            control = point(size, 0.76, 0.80)
            capture("purchase-order", lambda image: image.getpixel(point(size, 0.76, 0.78))[0] > 230)
            return control
        if name.startswith("inspect-slot-"):
            slot = int(name.rsplit("-", 1)[-1]) - 1
            if offer_faces is None:
                market = capture(f"{seat}-market", card_boxes)
                offer_faces = card_boxes(market)
            selected_face = offer_faces[slot]
            return center(selected_face)
        if name.startswith("sign-"):
            render = capture(f"{seat}-signing",
                             lambda image: annotation(image, selected_face, scale, GOLD, action=True))
            return annotation(render, selected_face, scale, GOLD, action=True)
        if name == "pass":
            render = capture(f"{seat}-ready-to-finish", lambda image: len(finish_buttons(image, scale)) == 2)
            return finish_buttons(render, scale)[0]
        if name.startswith("open-dugout-"):
            capture("dugout-matchup", matchup)
            return point(size, 0.19 if name.endswith("-sf") else 0.81, 0.72)
        if name == "close-dugout":
            capture("public-dugout", dugouts)
            return point(size, 0.50, 0.87)
        if name == "start-game-two":
            capture("start-game-two", matchup)
            return point(size, 0.50, 0.68)
        if name.startswith("glove-hit-box-"):
            number = int(name[-1])
            before = None if number == 1 else 1
            capture(f"before-preview-{number}", lambda image: selection_is(image, before))
            return choice_point(size, number)
        if name == "commit-glove-target":
            capture("before-use-glove", lambda image: selection_is(image, 2))
            return choice_point(size, 2, commit=True)

        current = position(save)
        club = current["clubs"][0 if seat == "sf" else 1]
        identity = name.split("-", 1)[1]
        if current["phase"] == "Demotion":
            if entry.kind == "component":
                index = next(i for i, card in enumerate(club["in_play"]) if card["identity"] == identity)
                opened = capture(f"{seat}-eligible", pending_demotion)
                demotion_market = [(box, opened.crop(box)) for box in secondary_offer_faces(opened)]
                selected_face = card_boxes(opened)[index]
                return center(selected_face)
            prepared = capture(f"{seat}-send-to-minors",
                               lambda image: demotion_action(image, selected_face, scale))
            require(all(ImageChops.difference(face, prepared.crop(box)).getbbox() is None
                        for box, face in demotion_market),
                    "Demotion help must leave all six replenished offers completely visible")
            return demotion_action(prepared, selected_face, scale)

        require(name.startswith(("inspect-", "play-")), f"No physical route to {name}")
        hand = club["hand"]
        index = next(i for i, card in enumerate(hand) if card["identity"] == identity)
        face = hand_faces[index]
        if entry.kind == "component":
            render = capture(f"{seat}-hand", lambda image: faces_visible(image, hand_faces[:len(hand)], stadium_scale))
            check_park(render, park)
            return center(face)
        render = capture(f"{seat}-play", lambda image: action_span(image, face, stadium_scale))
        start, end = action_span(render, face, stadium_scale)
        return center(face)[0], (start + end) // 2

    with ExitStack() as exits:
        windows = {"sf": open_window(exits, "sf", "sf-desktop")}
        if size == SIZES[0]:
            windows["ny"] = open_window(exits, "ny", "ny-desktop")
            _, window, bounds = windows["ny"]
            xdotool("windowraise", window, "windowfocus", "--sync", window)
            offer = point(size, 0.32, 0.27)
            move(window, offer)
            capture_when(bounds, evidence / "ny-revenue.png",
                         lambda image: image.getpixel((304, 313)) == (64, 94, 99))
            before = file_digest(save)
            click(window, offer)
            capture_when(bounds, evidence / "ny-revenue-offer.png", panel_visible)
            require(file_digest(save) == before, "Reading a revenue-screen offer changed the save")
        live_glove = None
        for step, entry in enumerate(trace, 1):
            if entry.seat not in windows:
                for game, window, _ in windows.values():
                    close(game, window)
                windows = {entry.seat: open_window(exits, entry.seat, f"{step:02}-{entry.seat}-desktop")}
            game, window, bounds = windows[entry.seat]
            xdotool("windowraise", window, "windowfocus", "--sync", window)

            def capture(name, ready):
                return capture_when(bounds, evidence / f"{step:02}-{name}.png", ready)

            before = file_digest(save)
            target = aim(entry, capture)
            if entry.name == "start-game-two" and size == SIZES[0]:
                _, observer, observed_bounds = windows["ny"]
                capture_when(observed_bounds, evidence / "ny-dugout-matchup.png", matchup)
                click(observer, point(size, 0.19, 0.72))
                capture_when(observed_bounds, evidence / "ny-dugout-before-start.png", dugouts)
                require(file_digest(save) == before, "Reading SF's Dugout changed the save")
                xdotool("windowraise", window, "windowfocus", "--sync", window)
            if entry.name.startswith("glove-hit-box-") or entry.name == "commit-glove-target":
                click(window, target)
            else:
                press(window, target)
            label = f"{entry.seat.upper()} / {entry.label}"
            if entry.kind == "choice":
                wait_for(lambda: file_digest(save) != before, label)
            require(file_digest(save) == entry.save_digest,
                    f"{label} ({entry.name}) diverged from the manager replay")

            if entry.name.startswith("open-dugout-"):
                capture(f"opened-{entry.name.removeprefix('open-')}", dugouts)
            if entry.name.startswith("inspect-slot-"):
                capture(f"{entry.seat}-offer-{entry.name.rsplit('-', 1)[-1]}",
                        lambda image: annotation(image, selected_face, scale, PAPER))
            if entry.name.startswith("sign-"):
                offer_faces = None
            if entry.seat == "sf" and entry.name.startswith("sign-"):
                capture("foxx-refill", pending_demotion)
                close(game, window)
                windows["sf"] = open_window(exits, "sf", "sf-desktop-resumed")
                _, _, resumed = windows["sf"]
                capture_when(resumed, evidence / "sf-resumed-pending-demotion.png", pending_demotion)
                require(file_digest(save) == entry.save_digest, "Restart changed the pending demotion")
            if entry.name == "play-FA-065":
                pending = capture("glove-pending", lambda image: selection_is(image, None))
                live_glove = pending.crop(park)
            if entry.name.startswith("glove-hit-box-"):
                number = int(entry.name[-1])
                capture(f"glove-preview-{number}-pressed", lambda image: selection_is(image, number))
                move(window, (5, 4))
                preview = capture(f"glove-preview-{number}", lambda image: selection_is(image, number))
                require(ImageChops.difference(live_glove, preview.crop(park)).getbbox() is None,
                        "Previewing a Glove outcome changed the live stadium")

        for seat in ("sf", "ny"):
            if seat not in windows:
                for game, window, _ in windows.values():
                    close(game, window)
                windows = {seat: open_window(exits, seat, f"{seat}-desktop-committed")}
            _, window, bounds = windows[seat]
            xdotool("windowraise", window, "windowfocus", "--sync", window)
            resolved = capture_when(bounds, evidence / f"{seat}-committed.png",
                                    lambda image: no_choices(image) and pawn_at(image, 1))
            check_park(resolved, park)
            require(file_digest(save) == trace[-1].save_digest, "The resolved public view changed the save")
        for game, window, _ in windows.values():
            close(game, window)


def smoke(executable, assets, evidence):
    starting_bytes = FIXTURE.read_bytes()
    executable_digest = file_digest(executable)
    fixture_digest = hashlib.sha256(starting_bytes).hexdigest()
    (evidence / "inputs.sha256").write_text(
        f"{executable_digest}  {executable}\n{fixture_digest}  starting-position.json\n",
    )
    (evidence / "starting-position.json").write_bytes(starting_bytes)
    agent_save = evidence / "agent-series.json"
    agent_save.write_bytes(starting_bytes)
    trace = []
    try:
        replay_managers(executable, assets, evidence, agent_save, trace)
    finally:
        (evidence / "controls.json").write_text(json.dumps([asdict(entry) for entry in trace], indent=2) + "\n")
    print("The two JSON-lines managers completed the game-two replay.", flush=True)
    for size in SIZES:
        directory = evidence / f"{size[0]}x{size[1]}"
        directory.mkdir()
        desktop_save = directory / "desktop-series.json"
        desktop_save.write_bytes(starting_bytes)
        replay_desktop(executable, assets, directory, desktop_save, trace, size)
        require(desktop_save.read_bytes() == agent_save.read_bytes(), f"X11 at {size} saved a different result")
        print(f"The full X11 replay at {size[0]} × {size[1]} matched the manager save byte for byte.", flush=True)
    require(file_digest(executable) == executable_digest, "The game executable changed during the replay")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--assets", type=Path, help="Path to the audited owner-local collection")
    parser.add_argument("--executable", type=Path, help="Use a specific built executable")
    args = parser.parse_args()
    evidence = Path(tempfile.mkdtemp(prefix="baseball-native-game-two."))
    print(f"Native game-two evidence in {evidence}", flush=True)
    try:
        check_display("scripts/native-buy-round --game-two", SIZES[-1])
        executable = args.executable.resolve() if args.executable else build()
        smoke(executable, args.assets, evidence)
    except (OSError, subprocess.SubprocessError, RuntimeError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
