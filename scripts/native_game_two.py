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

from native_buy_round import (
    GAME_SIZE, build, capture, capture_when, check_display, click, close,
    launch, position, require, wait_for, xdotool,
)
from native_glove import (
    GOLD, OBSERVER_X, PAPER, blue_pawn, capture_review, focus, selection_is,
)


FIXTURE = Path("tests/fixtures/expert-game-two-start.json")


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
def manager(executable, save, assets, evidence, transcript, seat):
    command = [executable, "--save", str(save), "--agent-seat", seat]
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

        choose_sf("choice", "buy-first-sf")
        choose_sf("component", "inspect-slot-2")
        purchase(choose_sf, "FA-065", "ST-004", "FA-067")
        closed = purchase(choose_ny, "FA-067", "ST-056", "FA-068")
        require(
            closed["balances"] == [0, 0] and [club["active"] for club in closed["rosters"]] == [15, 15],
            "The Buy Round must end with 15 active players per team and zero balance",
        )
        clubs = position(save)["clubs"]
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
        observer = observe(ny)["observation"]
        field = committed["field"]
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


def replay_desktop(executable, assets, evidence, save, trace):
    with ExitStack() as exits:
        windows = {
            seat: exits.enter_context(launch(
                executable, save, assets, evidence / f"{seat}-desktop.log",
                seat=seat, origin=(x, 0),
            ))
            for seat, x in (("sf", 0), ("ny", OBSERVER_X))
        }
        for entry in trace:
            _, window, table = windows[entry.seat]
            before = save.read_bytes()
            focus(window)
            click(window, entry.point)
            focus(window)
            label = f"{entry.seat.upper()} / {entry.label}"
            if entry.kind == "choice":
                wait_for(lambda: save.read_bytes() != before, label)
            else:
                require(save.read_bytes() == before, f"{label} changed the save during inspection")
            actual_digest = file_digest(save)
            require(
                actual_digest == entry.save_digest,
                f"{label} ({entry.name}) produced save SHA-256 {actual_digest}, "
                f"expected {entry.save_digest}",
            )

            if entry.name == "start-game-two":
                capture(table, evidence / "desktop-sf-hand.png")
            if entry.name == "play-FA-065":
                capture_review(table, evidence / "desktop-glove-pending.png", lambda im: selection_is(im, None))
            if entry.name.startswith("glove-hit-box-"):
                box = int(entry.name.rsplit("-", 1)[1])
                capture_review(table, evidence / f"desktop-preview-{box}.png", lambda im: selection_is(im, box))

        for seat, (game, window, table) in windows.items():
            focus(window)
            capture_review(table, evidence / f"desktop-{seat}-committed.png", lambda im: blue_pawn(im, (703, 275)))
            close(game, window)


def check_minimum_window(executable, assets, evidence):
    save = evidence / "minimum-window-series.json"
    shutil.copyfile(evidence / "ny-hand-position.json", save)
    bounds = (0, 0, 1080, 720)
    with launch(
        executable, save, assets, evidence / "minimum-window.log", seat="ny", origin=(0, 0),
    ) as (game, window, _):
        xdotool("windowsize", "--sync", window, 1080, 720)
        focus(window)
        capture_when(bounds, evidence / "minimum-hand.png", lambda im: im.getpixel((1055, 700)) == PAPER)
        before = save.read_bytes()
        click(window, (268, 439))
        focus(window)
        capture_when(bounds, evidence / "minimum-hand-selected.png", lambda im: im.getpixel((400, 597)) == GOLD)
        require(save.read_bytes() == before, "Inspecting Maris changed the save in the minimum window")
        click(window, (710, 597))
        wait_for(lambda: save.read_bytes() != before, "playing Maris in the 1080×720 window")
        require(position(save)["clubs"][1]["in_play"][-1]["identity"] == "FA-067",
                "The minimum window must play Maris")
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
    desktop_save = evidence / "desktop-series.json"
    agent_save.write_bytes(starting_bytes)
    desktop_save.write_bytes(starting_bytes)
    trace = []
    try:
        replay_managers(executable, assets, evidence, agent_save, trace)
    finally:
        (evidence / "controls.json").write_text(json.dumps([asdict(entry) for entry in trace], indent=2) + "\n")
    print("The two JSON-lines managers completed the game-two replay.", flush=True)
    replay_desktop(executable, assets, evidence, desktop_save, trace)
    require(desktop_save.read_bytes() == agent_save.read_bytes(), "X11 and JSON-lines saved different results")
    check_minimum_window(executable, assets, evidence)
    require(file_digest(executable) == executable_digest, "The game executable changed during the replay")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--assets", type=Path, help="Path to the audited owner-local collection")
    args = parser.parse_args()
    evidence = Path(tempfile.mkdtemp(prefix="baseball-native-game-two."))
    print(f"Native game-two evidence in {evidence}", flush=True)
    try:
        check_display("scripts/native-buy-round --game-two", (OBSERVER_X + GAME_SIZE[0], GAME_SIZE[1]))
        smoke(build(), args.assets, evidence)
    except (OSError, subprocess.SubprocessError, RuntimeError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    print("The native replay matched the JSON-lines save byte for byte. Maris played at 1080×720.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
