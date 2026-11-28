"""Verify signing, demotion and the buying handoff through both manager interfaces."""

import argparse
from contextlib import ExitStack
from pathlib import Path
import shutil
import sys
import tempfile
import time

from PIL import Image, ImageChops, ImageStat

from native_buy_round import build, check_display, close, launch, position, require, wait_for, xdotool
from native_game_two import FIXTURE, activate, file_digest, manager, observe
from native_market import (
    FOCUS, FRAME, GOLD, SIZES, annotation, capture_client, card_boxes, client_bounds, move, row_runs,
)


def control(reply, *, component=None, choice=None):
    return next((item for item in reply["table"]["controls"]
                 if item["component"] == component and item["choice"] == choice), None)


def refuse(request, revision, choice, message):
    try:
        request("activate", "activated", revision=revision, choice=choice)
    except RuntimeError as error:
        require(message in str(error), str(error))
        return
    raise RuntimeError(f"The manager incorrectly activated {choice}.")


def expected_demote(card):
    return f"demote-{card['identity']}-{card['instance']}"


def choose_demotion(request, seat, save, card, *, keyboard=False, reject_repeated=False):
    before = observe(request)
    captured = save.read_bytes()
    component = f"inspect-{card['identity']}"
    activate(request, seat, [], save, "component", component)
    reading = observe(request)
    require(reading["revision"] == before["revision"] and save.read_bytes() == captured
            and reading["table"]["market_inspection"]["zone"] == "In Play",
            "Reading a public In Play face must preserve the pending decision")
    revision = reading["revision"]
    choice = expected_demote(card)
    action = control(reading, choice=choice)
    require(action and action["on_screen"] and action["label"].startswith("Send "),
            "Only the chosen player's nearby Send to Minors should be actionable")
    if keyboard:
        help_focus = request("key", "pressed", revision=revision, key="Tab")
        require(help_focus["table"]["focus"]["component"].startswith("in-play-help-"),
                "Tab must move through the In Play explanation")
        action_focus = request("key", "pressed", revision=revision, key="Tab")
        require(action_focus["table"]["focus"]["choice"] == choice,
                "Tab must move from the explanation to the matching demotion")
        committed = request("key", "pressed", revision=revision, key="Enter", focused=choice)
    else:
        committed = request("activate", "activated", revision=revision, choice=choice)
    require(committed["table"]["cards_in_motion"] and not committed["settled"],
            "A committed demotion must have a visible card transfer")
    if keyboard:
        skipped = request("key", "pressed", revision=committed["revision"], key="Space")
        require(not skipped["table"]["cards_in_motion"] and skipped["settled"],
                "Space must finish the committed transfer immediately")
    landed = observe(request)
    view = landed["observation"]
    acting_seat = {"sf": "san_francisco", "ny": "new_york"}[seat]
    roster = next(club for club in view["rosters"] if club["seat"] == acting_seat)
    require(view["events"][-1] == {"event": "demoted", "seat": acting_seat, "card": card}
            and roster["active"] == 15 and roster["minor_leagues"] == 1
            and len(roster["in_play"]) == 5 and card not in [p["card"] for p in roster["in_play"]],
            "Demotion must remove the exact player from the active roster once")
    if reject_repeated:
        after = save.read_bytes()
        refuse(request, landed["revision"], choice, "unavailable")
        require(save.read_bytes() == after and observe(request)["observation"] == view,
                "Repeated demotion must preserve the committed position")
    return landed


def review_managers(executable, assets, evidence):
    save = evidence / "manager.json"
    pending_save = evidence / "pending.json"
    shutil.copyfile(FIXTURE, save)
    with ExitStack() as exits:
        transcript = exits.enter_context((evidence / "manager.jsonl").open("w"))
        sf = exits.enter_context(manager(executable, save, assets, evidence, transcript, "sf"))
        ny = exits.enter_context(manager(executable, save, assets, evidence, transcript, "ny"))

        before = activate(sf, "sf", [], save, "choice", "buy-first-sf")
        sign = "sign-5-28"
        commit_before = save.read_bytes()
        waiting = observe(ny)
        require(waiting["observation"]["decision"] is None and not control(waiting, choice=sign),
                "The waiting manager cannot sign before its turn")

        activate(sf, "sf", [], save, "component", "inspect-slot-5")
        reading = observe(sf)
        require(reading["table"]["market_inspection"]["budget"] == "$6 − $4 = $2 left"
                and control(reading, choice=sign)["label"] == "Sign Frankie Foxx for $4"
                and save.read_bytes() == commit_before,
                "The accessible signing must advertise Foxx's price and $2 remaining")
        signing = sf("activate", "activated", revision=reading["revision"], choice=sign)
        require(signing["table"]["cards_in_motion"], "Signing should play both committed transfers")
        pending = observe(sf)
        signed = pending["observation"]
        replacement = signed["offers"][4]["player"]["card"]
        foxx = before["offers"][4]["player"]["card"]
        require(signed["balances"] == [2, 7] and signed["rosters"][0]["active"] == 16
                and signed["free_agent_deck"] == before["free_agent_deck"] - 1
                and replacement["identity"] == "FA-067"
                and signed["offers"][:4] == observe(ny)["observation"]["offers"][:4]
                and [item["player"] for item in signed["offers"][:4] + signed["offers"][5:]]
                    == [item["player"] for item in before["offers"][:4] + before["offers"][5:]]
                and signed["events"][-1] == {
                    "event": "signed", "seat": "san_francisco", "slot": 5,
                    "card": foxx, "paid": 4, "replacement": replacement,
                },
                "Foxx must be the only signing, with Maris replacing him in slot five")
        eligible = signed["decision"]["eligible"]
        faces = [control(pending, component=f"inspect-{card['identity']}") for card in eligible]
        market = [control(pending, component=f"inspect-slot-{slot}") for slot in range(1, 7)]
        require(len(faces) == 6 and all(face and face["on_screen"] for face in faces),
                "All six eligible demotions need visible faces")
        require(all(face and face["on_screen"] for face in market),
                "Demotion must preserve the complete market")
        require(len({round(face["position"][1]) for face in faces}) == 1
                and min(b["position"][1] for b in market) > faces[0]["position"][1],
                "An In Play row needs visual priority over the market during demotion")
        focus = sf("key", "pressed", revision=pending["revision"], key="Tab")
        require(focus["table"]["focus"]["component"] == faces[0]["component"],
                "Keyboard travel must start with the first eligible player")
        waiting = observe(ny)
        require(waiting["observation"]["decision"] is None
                and waiting["observation"]["buyer"] == "san_francisco"
                and waiting["observation"]["balance"] == 2
                and waiting["observation"]["events"] == signed["events"],
                "NY must see SF's active decision, funds and history while waiting")
        pending_bytes = save.read_bytes()
        activate(ny, "ny", [], save, "component", "inspect-slot-5")
        refill = observe(ny)
        require(refill["table"]["market_inspection"]["budget"] == "$5 short"
                and all(not (item["choice"] or "").startswith("sign-") for item in refill["table"]["controls"])
                and save.read_bytes() == pending_bytes,
                "NY can read the $5 shortfall on Maris during SF's demotion without signing")
        shutil.copyfile(save, pending_save)
        for name, seat, card in (
            ("waiting-seat", "ny", eligible[0]),
            ("ineligible-card", "sf", {"identity": "ST-050", "instance": 107}),
        ):
            case = evidence / name
            case.mkdir()
            with manager(executable, save, assets, case, transcript, seat) as refusing:
                refuse(refusing, observe(refusing)["revision"], expected_demote(card), "unavailable")
        require(save.read_bytes() == pending_bytes, "An ineligible manager or card changed the save")

        landed = choose_demotion(sf, "sf", save, eligible[1], keyboard=True)
        require(landed["observation"]["decision"] == {"decision": "sign", "affordable": []},
                "SF has $2 left and no affordable signing")
        activate(sf, "sf", [], save, "choice", "pass")
        ny_turn = observe(ny)
        sf_waiting = observe(sf)
        require(ny_turn["observation"]["decision"]["decision"] == "sign"
                and sf_waiting["observation"]["decision"] is None
                and ny_turn["observation"]["balances"] == sf_waiting["observation"]["balances"] == [0, 7]
                and ny_turn["observation"]["balance"] == sf_waiting["observation"]["balance"] == 7
                and ny_turn["observation"]["events"][-1] == {
                    "event": "passed", "seat": "san_francisco", "unspent": 2,
                },
                "Both managers must see SF's $2 expire and NY's separate $7 decision")

        before_ny = ny_turn["revision"]
        activate(ny, "ny", [], save, "component", "inspect-slot-5")
        ny_choice = f"sign-5-{replacement['instance']}"
        activate(ny, "ny", [], save, "choice", ny_choice)
        updated = observe(sf)
        observe(ny)
        require(updated["observation"]["offers"][4]["player"]["name"] == "Babe Bench"
                and updated["observation"]["events"][-1]["card"] == replacement,
                "Another window must see Maris signed and Bench replenish the same slot")
        ny_eligible = observe(ny)["observation"]["decision"]["eligible"]
        choose_demotion(ny, "ny", save, ny_eligible[1])
        expected_history = [
            "SF chose SF to buy first.",
            "SF signed Frankie Foxx for $4. Mickey Maris entered the same slot.",
            "SF sent Rookie Natural ($2 revenue) to the Minor Leagues.",
            "SF finished buying. $2 expired.",
            "NY signed Mickey Maris for $7. Babe Bench entered the same slot.",
            "NY sent Veteran Natural ($1 revenue) to the Minor Leagues.",
        ]
        for request, seat in ((sf, "sf"), (ny, "ny")):
            before = observe(request)
            activate(request, seat, [], save, "component", "round-history")
            history = observe(request)
            require(control(history, component="close-history")
                    and history["table"]["round_history"] == expected_history
                    and history["observation"] == before["observation"],
                    "Round history must present the committed events without making another choice")

        committed = save.read_bytes()
        refuse(sf, before_ny, ny_choice, "Outdated")
        require(save.read_bytes() == committed, "A stale signing after another manager advanced changed the save")

        for index, card in enumerate(eligible):
            case = evidence / f"demote-{index + 1}"
            case.mkdir()
            save = case / "series.json"
            shutil.copyfile(pending_save, save)
            with manager(executable, save, assets, case, transcript, "sf") as reopened:
                view = observe(reopened)
                require(view["observation"] == signed and not view["table"]["cards_in_motion"],
                        "Reopening a pending demotion must show the same settled, exact position")
                choose_demotion(reopened, "sf", save, card, reject_repeated=True)


def center(box):
    left, top, right, bottom = box
    return (left + right) // 2, (top + bottom) // 2


def secondary_offer_faces(image):
    minimum = image.width // 19
    borders = []
    for y in range(round(image.height * 0.53), round(image.height * 0.83)):
        runs = row_runs(image, y, FRAME, minimum)
        if len(runs) == 6:
            borders.append((y, runs))
    if len(borders) < 2:
        return None
    top, runs = borders[0]
    bottom, _ = borders[-1]
    if bottom - top <= image.height / 8:
        return None
    return [(left, top, right, bottom) for left, right in runs]


def secondary_offers(image):
    return bool(secondary_offer_faces(image))


def demotion_action(image, face, scale, color=GOLD):
    left, _, right, bottom = face
    for y in range(bottom + round(12 * scale), bottom + round(51 * scale)):
        spans = row_runs(image, y, color, (right - left) // 2, left, right)
        if spans:
            start, end = spans[0]
            return (start + end) // 2, y + round(8 * scale)
    return None


def finish_buttons(image, scale):
    return [
        ((left + right) // 2, round(88 * scale))
        for left, right in row_runs(image, round(76 * scale), (235, 214, 176), round(40 * scale))
    ]


def press(window, point):
    move(window, point)
    xdotool("sleep", "0.12", "mousedown", 1, "sleep", "0.09", "mouseup", 1)


def capture_motion(frame, evidence, stem, size, *, skip=False):
    # Capture the real client while the card is still moving. A skipped
    # transfer settles both the visible piles and the revision already saved.
    moving = frame()
    moving.save(evidence / f"{stem}-initial.png")
    time.sleep(0.90 if not skip else 0.32)
    rest = frame()
    rest.save(evidence / f"{stem}-settled.png")
    width, height = size
    region = (width // 14, height // 8, width - width // 14, height - height // 10)
    diff = ImageChops.difference(moving.crop(region), rest.crop(region))
    if not skip:
        require(diff.getbbox() is not None, f"{stem} must visibly change while the card moves")
    return rest


def check_face_down(image, size, seat):
    boxes = card_boxes(image)
    require(boxes, "Six offers must return to the complete Buy Row after demotion")
    depth = min(size[0] / 1080, size[1] / 720)
    midpoints = {
        (1080, 720): {"sf": (91, 460), "ny": (615, 984)},
        (3840, 2160): {"sf": (309, 1644), "ny": (2181, 3516)},
    }
    lineup, minors = midpoints[size][seat]

    def center_pattern(x, y, half_width, half_height):
        bounds = (round(x - half_width * depth), round(y - half_height * depth),
                  round(x + half_width * depth), round(y + half_height * depth))
        return image.crop(bounds).resize((23, 31), Image.Resampling.BILINEAR)

    bottom = boxes[0][3]
    deck = center_pattern(64.5 * depth, bottom - 69 * depth, 25, 35)
    for x, label in ((lineup, "Lineup"), (minors, "Minor Leagues")):
        pile = center_pattern(x, bottom + 185 * depth, 26, 36.5)
        difference = ImageStat.Stat(ImageChops.difference(deck, pile))
        require(sum(difference.mean) / 3 < 28,
                f"{seat} {label} must use the visible back of the Free Agent Deck")


def review_window(executable, assets, evidence, size, seat, save, *, keyboard, skip):
    width, height = size
    scale = min(width / 1080, height / 720)
    index = 0 if seat == "sf" else 1
    identity, replacement = (("FA-065", "FA-067") if index == 0 else ("FA-067", "FA-068"))
    with launch(executable, save, assets, evidence / f"{seat}-desktop.log", seat=seat, origin=(0, 0)) as (
        game, window, _,
    ):
        xdotool("windowsize", "--sync", window, width, height)
        wait_for(lambda: client_bounds(window) == (0, 0, width, height), f"{seat} client at {size}")
        move(window, (width // 2, 40))

        def frame():
            return capture_client((0, 0, width, height))

        face = wait_for(lambda: card_boxes(frame()), f"{seat} visible Press Box market")[4]
        frame().save(evidence / f"{seat}-market.png")
        if keyboard:
            press(window, center(face))
        else:
            move(window, center(face))
        action = wait_for(lambda: annotation(frame(), face, scale, GOLD, action=True),
                          f"{seat} autograph Sign action")
        frame().save(evidence / f"{seat}-sign-offer.png")
        before = position(save)
        if keyboard:
            xdotool("key", "--delay", 140, "Tab", "Tab")
            wait_for(lambda: annotation(frame(), face, scale, FOCUS, action=True), "Sign keyboard focus")
            xdotool("key", "Return", "Return")
        else:
            press(window, action)
        signed = wait_for(lambda: (current if (current := position(save)) != before else None),
                          f"{seat} signing committed")
        frame().save(evidence / f"{seat}-signing-in-flight.png")
        require(signed["row"][4]["identity"] == replacement and signed["clubs"][index]["lineup"][-1]["identity"] == identity
                and signed["balances"] == ([2, 7] if index == 0 else [0, 0]),
                "Native signing changed the wrong player, slot or funds")
        if not keyboard:
            press(window, action)
            require(position(save) == signed, "A double/signing click committed a second action")
            move(window, (width // 2, 40))
        def pending_faces():
            image = frame()
            return card_boxes(image) if secondary_offers(image) else None

        eligible = wait_for(pending_faces, "priority demotion faces with the complete market")
        require(len(eligible) == 6, "Six visible demotions must fit above the market")
        frame().save(evidence / f"{seat}-pending-demotion.png")
        selected = 0 if keyboard else 1
        card = signed["clubs"][index]["in_play"][selected]
        face = eligible[selected]
        if keyboard:
            xdotool("key", "--delay", 140, "Tab", "Tab", "Tab")
            wait_for(lambda: demotion_action(frame(), face, scale, FOCUS), "demotion keyboard focus")
            xdotool("key", "Return", "Return")
        else:
            move(window, center(face))
            action = wait_for(lambda: demotion_action(frame(), face, scale), "nearby Send to Minors")
            frame().save(evidence / f"{seat}-send-to-minors.png")
            press(window, action)
        if skip:
            # The committing key changes the save first. Wait for that board
            # before sending Space to the transfer that belongs to it.
            wait_for(lambda: len(finish_buttons(frame(), scale)) == 2, "the committed demotion board")
            xdotool("key", "space")
        demoted = wait_for(lambda: (current if (current := position(save)) != signed else None),
                           f"demotion of the exact {seat} player")
        landed = capture_motion(frame, evidence, f"{seat}-demotion", size, skip=skip)
        check_face_down(landed, size, seat)
        require(demoted["clubs"][index]["minor_leagues"] == [card]
                and card not in demoted["clubs"][index]["in_play"]
                and sum(len(demoted["clubs"][index][zone]) for zone in ("lineup", "in_play")) == 15,
                "The native demotion did not put the exact selected player in the Minor Leagues")
        buttons = wait_for(lambda: (buttons if len(buttons := finish_buttons(frame(), scale)) == 2 else None),
                           "Finish buying and Round history")
        press(window, buttons[1])
        time.sleep(0.30)
        frame().save(evidence / f"{seat}-history.png")
        require(position(save) == demoted, "Opening native history changed the decision")
        close(game, window)
    return signed, demoted


def review_recovery_and_pass(executable, assets, evidence, save, size, seat, signed):
    width, height = size
    scale = min(width / 1080, height / 720)
    recovery = evidence / f"{seat}-recovered.json"
    with launch(executable, recovery, assets, evidence / f"{seat}-recovery.log",
                seat=seat, origin=(0, 0), args=("--recover-from", str(save))) as (game, window, _):
        xdotool("windowsize", "--sync", window, width, height)
        bounds = (0, 0, width, height)
        wait_for(lambda: secondary_offers(capture_client(bounds)), "restored pending demotion")
        capture_client(bounds).save(evidence / f"{seat}-recovery.png")
        require(position(recovery) == signed, "Recovery must open the prior pending demotion exactly")
        close(game, window)
    if seat == "ny":
        return
    with launch(executable, save, assets, evidence / "sf-pass.log", seat=seat, origin=(0, 0)) as (
        game, window, _,
    ):
        xdotool("windowsize", "--sync", window, width, height)
        bounds = (0, 0, width, height)
        buttons = wait_for(lambda: (items if len(items := finish_buttons(capture_client(bounds), scale)) == 2 else None),
                           "restored Finish buying")
        press(window, buttons[0])
        wait_for(lambda: position(save)["balances"] == [0, 7], "SF's $2 expiry")
        time.sleep(0.40)
        capture_client(bounds).save(evidence / "sf-waiting.png")
        close(game, window)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--assets", type=Path)
    parser.add_argument("--executable", type=Path)
    args = parser.parse_args()
    evidence = Path(tempfile.mkdtemp(prefix="baseball-native-signing."))
    print(f"Signing evidence in {evidence}", flush=True)
    try:
        executable = args.executable.resolve() if args.executable else build()
        digest = file_digest(executable)
        check_display("scripts/native-buy-round --signing", SIZES[-1])
        review_managers(executable, args.assets, evidence)
        for size in SIZES:
            directory = evidence / f"{size[0]}x{size[1]}"
            directory.mkdir()
            save = directory / "series.json"
            shutil.copyfile(FIXTURE, save)
            with (directory / "order.jsonl").open("w") as transcript, manager(
                executable, save, args.assets, directory, transcript, "sf",
            ) as sf:
                activate(sf, "sf", [], save, "choice", "buy-first-sf")
            for seat in ("sf", "ny"):
                keyboard = (seat == "sf") == (size == SIZES[-1])
                signed, _ = review_window(executable, args.assets, directory, size, seat, save,
                                          keyboard=keyboard, skip=keyboard)
                review_recovery_and_pass(executable, args.assets, directory, save, size, seat, signed)
        require(file_digest(executable) == digest, "The executable changed during the replay")
        (evidence / "inputs.sha256").write_text(
            f"{digest}  {executable}\n{file_digest(FIXTURE)}  {FIXTURE}\n")
    except (OSError, RuntimeError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    print("Signing, exact demotion, transfer and recovery checks passed through both interfaces.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
