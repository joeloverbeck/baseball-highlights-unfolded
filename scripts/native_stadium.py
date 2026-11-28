"""Verify the game-two stadium and hand through the manager seats and X11."""

import argparse
import json
from contextlib import ExitStack
from pathlib import Path
import tempfile
import time

from native_buy_round import build, check_display, click, close, launch, require, wait_for, xdotool
from native_game_start import HANDS, card_keys, named
from native_game_two import FIXTURE, activate, file_digest, manager, observe, purchase
from native_market import FOCUS, FRAME, GOLD, capture_client, client_bounds, move, row_runs


SIZES = (1080, 720), (3840, 2160)
BOARD = (21, 40, 47)


def refuse(request, revision, choice, reason):
    try:
        request("activate", "activated", revision=revision, choice=choice)
    except RuntimeError as error:
        require(reason in str(error), f"{choice} failed for the wrong reason: {error}")
    else:
        require(False, f"{choice} took effect when it should have been refused")


def hand_faces(reply, cards):
    faces = [named(reply, "component", f"inspect-{identity}") for identity, _ in cards]
    require(all(faces), "Every opening card needs a reachable face")
    return [tuple(face["position"]) for face in faces]


def check_grid(faces):
    top, bottom = faces[:3], faces[3:]
    require(
        top[0][1] == top[1][1] == top[2][1]
        and bottom[0][1] == bottom[1][1] == bottom[2][1] > top[0][1]
        and [card[0] for card in top] == [card[0] for card in bottom]
        and top[0][0] < top[1][0] < top[2][0],
        "The opening faces must form three aligned columns and two rows in hand order",
    )


def review_managers(executable, assets, evidence):
    save = evidence / "manager-series.json"
    save.write_bytes(FIXTURE.read_bytes())
    with ExitStack() as exits:
        transcript = exits.enter_context((evidence / "manager-transcript.jsonl").open("w"))
        sf, ny = [exits.enter_context(manager(executable, save, assets, evidence, transcript, team))
                  for team in ("sf", "ny")]

        def choose_sf(kind, name):
            return activate(sf, "sf", [], save, kind, name)

        def choose_ny(kind, name):
            return activate(ny, "ny", [], save, kind, name)

        choose_sf("choice", "buy-first-sf")
        purchase(choose_sf, "FA-065", "ST-004", "FA-067")
        purchase(choose_ny, "FA-067", "ST-056", "FA-068")
        choose_sf("choice", "start-game-two")
        (evidence / "visitor-opener.json").write_bytes(save.read_bytes())
        opening = observe(sf)
        observer = observe(ny)
        for index, reply in enumerate((opening, observer)):
            view = reply["observation"]
            require(
                view["standing"] == {"game": 2, "home": "new_york", "wins": [0, 1]}
                and [diamond["score"] for diamond in view["field"]["diamonds"]] == [0, 0]
                and view["acting_manager"] == "san_francisco",
                "Game runs, series wins, Home and the acting manager must be distinct",
            )
            require(card_keys(view["rosters"][index]["hand"]) == HANDS[index]
                    and "hand" not in view["rosters"][1 - index],
                    "Each manager must see their own exact hand and only their own hand")
        require(opening["table"]["stadium"]["prompt"] == "Play your first player"
                and observer["table"]["stadium"]["prompt"] == "San Francisco opens as Visitor"
                and observer["observation"]["decision"] is None
                and observer["table"]["hand_inspection"] is None,
                "The Home manager may watch the Visitor opener without seeing their choice")
        require(not any((control["component"] or "").startswith("inspect-")
                        for control in observer["table"]["controls"]),
                "Waiting must not expose face controls for the Visitor's hand")
        check_grid(hand_faces(opening, HANDS[0]))

        revision = opening["revision"]
        before = save.read_bytes()
        first = sf("key", "pressed", revision=revision, key="Tab")
        require((first["table"]["focus"] or {}).get("component") == "inspect-FA-065",
                "Tab must focus the first opening face")
        require([item["mark"] for item in first["table"]["hand_inspection"]["symbols"]] == ["S", "PH"],
                "Frankie Foxx must disclose only his printed Slow and PH symbols")
        help_focus = sf("key", "pressed", revision=revision, key="Tab")
        require(help_focus["table"]["focus"]["component"] == "hand-help-FA-065",
                "Tab must reach the disclosure directly from its source face")
        action_focus = sf("key", "pressed", revision=revision, key="Tab")
        require(action_focus["table"]["focus"]["choice"] == "play-FA-065",
                "Tab must reach the explicit Play control")
        reversed_focus = sf("key", "pressed", revision=revision, key="Shift+Tab")
        require(reversed_focus["table"]["focus"]["component"] == "hand-help-FA-065",
                "Reverse Tab must return to the same disclosure from Play")
        dismissed = sf("key", "pressed", revision=revision, key="Escape")
        require(dismissed["table"]["focus"]["component"] == "inspect-FA-065"
                and dismissed["table"]["hand_inspection"] is None
                and observe(sf)["table"]["hand_inspection"] is None,
                "Escape must retain the focused face and suppress its disclosure")
        next_face = sf("key", "pressed", revision=revision, key="Tab")
        require(next_face["table"]["focus"]["component"] == "inspect-ST-013"
                and [item["mark"] for item in next_face["table"]["hand_inspection"]["symbols"]] == ["A"],
                "Focus on a different face must rearm help with that player's symbols")
        require(next_face["table"]["hand_inspection"]["card"] == {"identity": "ST-013", "instance": 98},
                "The inspection must bind the full card, including its physical instance")
        refuse(sf, revision, "play-FA-065", "named control is unavailable")
        refuse(ny, revision, "play-ST-013", "named control is unavailable")
        require(save.read_bytes() == before and next_face["observation"] == opening["observation"],
                "Focus, dismissal and inaccessible card or seat requests must preserve the position")
        # A refusal deliberately disarms further input on that connection.
        resumed = evidence / "resumed-sf"
        resumed.mkdir()
        sf = exits.enter_context(manager(executable, save, assets, resumed, transcript, "sf"))
        choose_sf("component", "inspect-ST-013")
        choose_sf("choice", "play-ST-013")
        committed = save.read_bytes()
        response = observe(ny)
        waiting = observe(sf)
        for reply in (response, waiting):
            require(reply["table"]["stadium"]["viewport"] == opening["table"]["stadium"]["viewport"],
                    "The stadium must remain anchored after a legal play")
        require(response["table"]["stadium"]["prompt"] == "Choose a response to the threat"
                and waiting["table"]["stadium"]["prompt"] == "New York responds to the threat"
                and response["table"]["stadium"]["threat"] == "San Francisco's Veteran Natural threatens a single."
                and response["observation"]["field"] == waiting["observation"]["field"],
                "Both seats must see the public threat, with the next choice going to Home")
        require(waiting["observation"]["rosters"][0]["in_play"][-1]["card"]
                == {"identity": "ST-013", "instance": 98},
                "Play must use the selected physical Veteran Natural")
        refuse(sf, revision, "play-ST-013", "Outdated")
        require(save.read_bytes() == committed, "A stale play must leave the committed position intact")
        (evidence / "manager-after-play.json").write_bytes(committed)



def geometry(size):
    # Independent readings of the selected 1920 × 1000 match study.
    width, height = size
    s = min(width / 1920, height / 1000)
    x = (width - 1920 * s) / 2
    y = (height - 1000 * s) / 2

    def box(left, top, right, bottom):
        return tuple(round(value) for value in
                     (x + left * s, y + top * s, x + right * s, y + bottom * s))
    park = box(36, 245, 1070, 848)
    cards = [box(card_x, card_y, card_x + 222, card_y + 309)
             for card_y in (258, 633) for card_x in (1130, 1382, 1634)]
    return s, park, cards


def faces_visible(image, faces, scale):
    for left, top, right, _ in faces:
        if not any(row_runs(image, y, FRAME, (right - left) - round(22 * scale),
                            left, right) for y in range(top, top + round(3 * scale) + 1)):
            return False
    return True


def outline(image, face, scale, color):
    x = (face[0] + face[2]) // 2
    top = face[1]
    return any(image.getpixel((x, y)) == color
               for y in range(top - round(7 * scale), top))


def action_span(image, face, scale, color=GOLD):
    left, _, right, bottom = face
    x = left + (right - left) // 5
    y = range(bottom + round(17 * scale), bottom + round(55 * scale))
    painted = [row for row in y if image.getpixel((x, row)) == color]
    return (painted[0], painted[-1] + 1) if painted else None


def check_park(image, park):
    x, y, right, bottom = park
    mid_x, mid_y = (x + right) // 2, (y + bottom) // 2
    expected = (image.getpixel((x - 2, mid_y)), image.getpixel((mid_x, y - 2)),
                image.getpixel((right + 2, mid_y)), image.getpixel((mid_x, bottom + 2)))
    actual = (image.getpixel((x + 2, mid_y)), image.getpixel((mid_x, y + 2)),
              image.getpixel((right - 2, mid_y)), image.getpixel((mid_x, bottom - 2)))
    require(all(pixel == BOARD for pixel in expected) and all(pixel != BOARD for pixel in actual),
            f"The rendered stadium must fill the 1034 × 603 design bounds, sampled {expected} / {actual}")
    require(image.crop(park).resize((32, 16)).getcolors(16) is None,
            "The stadium art must be rendered inside its viewport")


def review_window(executable, assets, evidence, size, opener, committed):
    width, height = size
    s, park, faces = geometry(size)
    save = evidence / "series.json"
    save.write_bytes(opener)
    with launch(executable, save, assets, evidence / "desktop.log", seat="sf", origin=(0, 0)) as (
        game, window, _,
    ):
        xdotool("windowsize", "--sync", window, width, height)
        bounds = (0, 0, width, height)
        require(client_bounds(window) == bounds, f"Expected an actual {width} × {height} X11 client")
        frame = lambda: capture_client(bounds)
        move(window, (width - 5, height - 5))

        def rendered():
            image = frame()
            return image if faces_visible(image, faces, s) else None

        rest = wait_for(rendered, f"all six rendered faces in the {width} × {height} client")
        check_park(rest, park)
        rest.save(evidence / "at-rest.png")
        first = faces[0]
        xdotool("key", "Tab")
        wait_for(lambda: outline(frame(), first, s, FOCUS), "Tab focusing the first face")
        xdotool("key", "Tab")
        wait_for(lambda: outline(frame(), first, s, GOLD), "Tab moving from face to the disclosure")
        xdotool("key", "Tab")
        wait_for(lambda: action_span(frame(), first, s, FOCUS), "Tab reaching Play")
        xdotool("key", "shift+Tab")
        wait_for(lambda: not action_span(frame(), first, s, FOCUS), "reverse Tab returning to the disclosure")
        xdotool("key", "Escape")
        wait_for(lambda: not action_span(frame(), first, s), "Escape closing the disclosure")
        time.sleep(0.55)
        require(outline(frame(), first, s, FOCUS) and not action_span(frame(), first, s),
                "Escape must keep the face focused without reopening Play")
        xdotool("key", "Tab")
        wait_for(lambda: outline(frame(), faces[1], s, FOCUS)
                 and action_span(frame(), faces[1], s), "Tab reopening on a different face")

        points = []
        for index, face in enumerate(faces):
            left, top, right, bottom = face
            x = (left + right) // 2
            move(window, (x, (top + bottom) // 2))
            action_top, action_bottom = wait_for(
                lambda: action_span(frame(), face, s), f"Play under opening face {index + 1}",
            )
            require(abs((action_bottom - action_top) - 32 * s) <= 2,
                    f"The displayed Play control on face {index + 1} must be 32 design units")
            highlight = frame()
            visible_y = [y for y in range(bottom, action_top)
                         if highlight.getpixel((x, y)) in (GOLD, FOCUS)]
            require(visible_y and abs((action_top - visible_y[-1] - 1) - 14 * s) <= 2,
                    f"Face {index + 1} must have 14 visible design units between outline and Play")
            gap = (visible_y[-1] + action_top) // 2
            for y in (bottom - round(12 * s), bottom, gap):
                move(window, (x, y))
                time.sleep(0.17)
            time.sleep(0.42)
            require(action_span(frame(), face, s), "Play disappeared during slow pointer travel")
            click(window, (x, gap))
            require(save.read_bytes() == opener and action_span(frame(), face, s),
                    "Clicking the clearance must preserve the position and the open disclosure")
            aim = (x, (action_top + action_bottom) // 2)
            move(window, (x, action_top))
            time.sleep(0.17)
            require(action_span(frame(), face, s), "Play disappeared at the action edge")
            move(window, aim)
            time.sleep(0.16)
            points.append({"face": [left, top, right, bottom], "gap": [x, gap], "play": list(aim)})
            frame().save(evidence / f"face-{index + 1}-play.png")
            require(action_span(frame(), face, s), "The pointer could not reach Play")

        # Cross horizontally along the bridge from the farthest card to the help
        # while the other two faces in that row stay independently reachable.
        last = points[-1]
        x, y = last["gap"]
        move(window, (x, y))
        for fraction in (0.15, 0.35, 0.55, 0.75, 1):
            move(window, (round(x + (round(880 * s) - x) * fraction), y))
            time.sleep(0.12)
        require(action_span(frame(), faces[-1], s), "Pointer travel to the help lost the source face")
        frame().save(evidence / "beside-card-help.png")

        # The seat check above played the second card in this exact position.
        left, top, right, bottom = faces[1]
        move(window, ((left + right) // 2, (top + bottom) // 2))
        wait_for(lambda: action_span(frame(), faces[1], s), "the same Veteran Natural's Play")
        click(window, points[1]["play"])
        wait_for(lambda: save.read_bytes() != opener, "the Veteran Natural's legal opener")
        require(save.read_bytes() == committed, "The native and seat plays selected different cards")
        def waiting_frame():
            image = frame()
            return None if faces_visible(image, faces, s) else image

        waiting = wait_for(waiting_frame, "the public view after the Visitor commits")
        waiting.save(evidence / "visitor-played.png")
        check_park(waiting, park)
        left, top, right, bottom = park
        static = left + 2, top + 2, right - 2, (top + bottom) // 2
        require(rest.crop(static).tobytes() == waiting.crop(static).tobytes(),
                "The stadium background changed position across the play")
        (evidence / "geometry.json").write_text(
            json.dumps({"client": size, "stadium": park, "hand": points}, indent=2) + "\n"
        )
        close(game, window)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--assets", type=Path, help="The audited owner-local art collection")
    parser.add_argument("--binary", type=Path, help="Test an already built desktop binary")
    arguments = parser.parse_args()
    evidence = Path(tempfile.mkdtemp(prefix="baseball-native-stadium."))
    print(f"Stadium evidence in {evidence}", flush=True)
    check_display("scripts/native-stadium", SIZES[-1])
    executable = arguments.binary or build()
    digest = file_digest(executable)
    (evidence / "inputs.sha256").write_text(
        f"{digest}  {executable}\n{file_digest(FIXTURE)}  {FIXTURE}\n"
    )
    review_managers(executable, arguments.assets, evidence)
    opener = (evidence / "visitor-opener.json").read_bytes()
    committed = (evidence / "manager-after-play.json").read_bytes()
    for size in SIZES:
        capture = evidence / f"{size[0]}x{size[1]}"
        capture.mkdir()
        review_window(executable, arguments.assets, capture, size, opener, committed)
    require(file_digest(executable) == digest, "The executable changed during verification")
    print("Both seats and both X11 clients played the exact Visitor card with an anchored stadium.")


if __name__ == "__main__":
    main()
