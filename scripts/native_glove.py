"""Verify both Glove fixtures through the manager seats and ordinary X11 window."""

import argparse
from contextlib import ExitStack, contextmanager
import json
from pathlib import Path
from struct import iter_unpack
import subprocess
import sys
import tempfile

from PIL import ImageChops

from native_buy_round import (
    GAME_SIZE, TITLE as EXPERT_TITLE, build, capture_when, check_display, click, close, launch,
    require, wait_for, xdotool,
)


TITLE = "Baseball Highlights / Glove Review"
OBSERVER_X = GAME_SIZE[0] + 32
SIZES = (1080, 720), (3840, 2160)
PAPER = (235, 214, 176)
GOLD = (245, 199, 122)
BOARD = (21, 40, 47)
HELP = (247, 232, 204)
FRAME = (81, 107, 101)
FOCUS = (255, 242, 205)
FACE_KEYS = ("inspect-glove-batter", "inspect-glove-defender")


def design_point(size, x, y):
    width, height = size
    scale = min(width / 1920, height / 1000)
    return (
        round((width - 1920 * scale) / 2 + x * scale),
        round((height - 1000 * scale) / 2 + y * scale),
    )


def design_rect(size, left, top, right, bottom):
    return (*design_point(size, left, top), *design_point(size, right, bottom))


def choice_point(size, box, *, commit=False):
    return design_point(size, 1310 + (box - 1) * 392, 904 if commit else 742)


def focus(window):
    xdotool("windowraise", window, "windowfocus", "--sync", window)
    xdotool("mousemove", "--window", window, 5, 4)


def border_at(image, x, y, colors):
    x, y = design_point(image.size, x, y)
    return any(image.getpixel((x, y + offset)) in colors for offset in range(-2, 3))


def faces_visible(image):
    return all(border_at(image, x, 285, (FRAME, FOCUS, GOLD)) for x in (1240, 1586))


def capture_review(table, path, ready):
    return capture_when(table, path, lambda image: faces_visible(image) and ready(image))


def selection_is(image, selected):
    def pixel(x, y):
        return image.getpixel(design_point(image.size, x, y))

    for box, delta in ((1, 0), (2, 392)):
        if pixel(1140 + delta, 632) != PAPER:
            return False
        stroke = (GOLD,) if selected == box else (PAPER,)
        if not border_at(image, 1310 + delta, 620, stroke):
            return False
        if pixel(1140 + delta, 920) != (GOLD if selected == box else BOARD):
            return False
    return True


def no_choices(image):
    return all(image.getpixel(design_point(image.size, x, 638)) != PAPER for x in (1140, 1532))


def stadium_rendered(image):
    # The lower diamond paint and the bottom edge render after the UI during an X11 resize.
    lower_paint = image.getpixel(design_point(image.size, 420, 680))
    _, green, blue = image.getpixel(design_point(image.size, 800, 847.4))
    return min(lower_paint) > 75 and blue > 35 and abs(green - blue) <= 4


def blue_pawn(image, point):
    x, y = point
    samples = iter_unpack("BBB", image.crop((x - 5, y - 5, x + 6, y + 6)).tobytes())
    return sum(blue > green + 35 and green > red + 35 for red, green, blue in samples) >= 20


def pawn_at(image, base):
    # The independently authored studio background locates first and second.
    x, y = ((677., 679.46), (553., 584.12))[base - 1]
    return blue_pawn(image, design_point(image.size, x, y))


def control(reply, kind, name):
    return next((item for item in reply["table"]["controls"]
                 if item[kind] == name and item["on_screen"]), None)


def refused(request, kind, name, revision, reason):
    try:
        request("activate", "activated", revision=revision, **{kind: name})
    except RuntimeError as error:
        require(reason in str(error).casefold(), f"{name} failed for a different reason: {error}")
    else:
        require(False, f"{name} took effect when it should have been refused")


def prepare_pending(executable, assets, evidence, practice):
    from native_game_two import FIXTURE, activate, manager, observe, play, purchase

    save = evidence / "pending-source.json"
    args = ("--review-glove",) if practice else ()
    if not practice:
        save.write_bytes(FIXTURE.read_bytes())
    with ExitStack() as scope:
        log = scope.enter_context((evidence / "manager-setup.jsonl").open("w"))
        sf = scope.enter_context(manager(executable, save, assets, evidence, log, "sf", args=args))
        if not practice:
            ny = scope.enter_context(manager(executable, save, assets, evidence, log, "ny"))
            def choose(seat, request):
                return lambda kind, name: activate(request, seat, [], save, kind, name)

            sf_choose = choose("sf", sf)
            ny_choose = choose("ny", ny)
            sf_choose("choice", "buy-first-sf")
            purchase(sf_choose, "FA-065", "ST-004", "FA-067")
            purchase(ny_choose, "FA-067", "ST-056", "FA-068")
            sf_choose("choice", "start-game-two")
            play(sf_choose, "ST-013")
            play(ny_choose, "FA-067")
            play(sf_choose, "FA-065")
        require(observe(sf)["observation"]["field"]["glove_pending"],
                "The fixture must open at a pending Glove decision")
    return save.read_bytes()


def check_branch(executable, assets, evidence, practice, box, pending):
    from native_game_two import manager, observe

    save = evidence / "manager.json"
    save.write_bytes(pending)
    args = ("--review-glove",) if practice else ()
    with ExitStack() as scope:
        log = scope.enter_context((evidence / "manager.jsonl").open("w"))
        sf = scope.enter_context(manager(executable, save, assets, evidence, log, "sf", args=args))
        ny = scope.enter_context(manager(executable, save, assets, evidence, log, "ny", args=args))
        start, observer = observe(sf), observe(ny)
        view, other = start["observation"], observer["observation"]
        revision = start["revision"]
        require(view["field"] == other["field"] and other["decision"] is None,
                "NY must see the same public field and wait for SF")
        require(all("hand" not in reply["observation"]["rosters"][1 - index]
                    for index, reply in enumerate((start, observer))),
                "The public Glove matchup must preserve hand privacy")
        require(control(start, "choice", "commit-glove-target") is None,
                "Use Glove must wait for a selected outcome")
        require(control(observer, "component", "glove-hit-box-1") is None
                and control(observer, "component", "glove-hit-box-2") is None,
                "NY must have no Glove outcome control")

        defender = "Rookie Natural" if practice else "Frankie Foxx"
        marks = ["A"] if practice else ["S", "PH"]
        for request, initial in ((sf, start), (ny, observer)):
            for component, name, team, expected_marks in (
                (FACE_KEYS[0], "Mickey Maris", "new_york", ["A"]),
                (FACE_KEYS[1], defender, "san_francisco", marks),
            ):
                face = control(initial, "component", component)
                require(face and name in face["label"], f"{name} needs a public readable face")
                request("point", "pointed", revision=revision, component=component)
                read = observe(request)
                disclosure = read["table"]["glove_inspection"]
                require(disclosure and disclosure["name"] == name and disclosure["team"] == team
                        and [symbol["mark"] for symbol in disclosure["symbols"]] == expected_marks,
                        f"The public face must explain the actual symbols for {name}")
                require(read["observation"] == initial["observation"] and save.read_bytes() == pending,
                        "Public inspection changed the game")
                request("key", "pressed", revision=revision, key="Escape")
                require(observe(request)["table"]["glove_inspection"] is None,
                        "Escape must dismiss the public face help")

        expected = (4, 3) if practice else (1, 0)
        for target, hit in enumerate(("single", "home run"), 1):
            action = f"glove-hit-box-{target}"
            option = control(start, "component", action)
            label = option["label"] if option else ""
            landing = "Mickey Maris scores." if target == 1 else "Mickey Maris reaches first."
            survives = "home run" if target == 1 else "single"
            occupancy = "Bases empty" if target == 1 else "First / Mickey Maris (Average)"
            require(all(word in label for word in (
                f"Mickey Maris's {hit}", f"Survives: {survives}", landing,
                f"Projected  NY {expected[target - 1]}  /  SF {int(practice)}", occupancy,
            )), f"Preview {target} should explain this exact projection: {label}")
            if practice:
                require("Veteran Cyborg scores from second." in label,
                        "The practice preview must identify the runner scoring from second")
            preview = sf("activate", "activated", revision=revision, component=action)
            require(preview["observation"] == view and preview["revision"] == revision
                    and preview["table"]["stadium"]["viewport"] == start["table"]["stadium"]["viewport"]
                    and save.read_bytes() == pending, "The preview changed the live state")
            require(control(preview, "choice", "commit-glove-target") is not None,
                    "A preview must offer a separate Use Glove action")

        # Re-select the branch, then commit exactly once with its original revision.
        sf("activate", "activated", revision=revision, component=f"glove-hit-box-{box}")
        require(observe(ny)["observation"] == other, "SF's preview changed NY's view")
        committed = sf("activate", "activated", revision=revision, choice="commit-glove-target")
        seen = observe(ny)
        final = committed["observation"]
        field = final["field"]
        require(field == seen["observation"]["field"] and not field["glove_pending"],
                "Both seats must see the same resolved field")
        sf_diamond, ny_diamond = field["diamonds"]
        counters = [runner["card"]["identity"] if runner else None for runner in ny_diamond["bases"]]
        require((sf_diamond["score"], ny_diamond["score"], counters) == (
            int(practice), expected[box - 1], [None if box == 1 else "FA-067", None, None],
        ), "The selected branch produced the wrong runs or bases")
        require([(threat["hit"], threat["target"]["card"]["identity"]) for threat in field["threatened"]]
                == ([("single", "ST-001")] if practice else [("double", "FA-065")]),
                "The Glove player's own threat must wait for the response")
        events = final["events"][-3:]
        require([event["event"] for event in events] == ["glove_cancelled", "hit_resolved", "threats_placed"]
                and [event["threat"]["target"]["number"] for event in events[:2]] == [box, 3 - box],
                "The scorecard must identify one cancellation and the surviving hit")
        status = seen["table"]["stadium"]["recent_plays"]
        require(any("used Glove to cancel Mickey Maris's" in line for line in status)
                and any(line.startswith("Mickey Maris /") for line in status)
                and any(f"{defender} threatens" in line for line in status),
                "The resolved panel must show what happened and the next threat")
        require(seen["table"]["stadium"]["viewport"] == start["table"]["stadium"]["viewport"],
                "The viewport moved after resolution")
        committed_bytes = save.read_bytes()
        refused(sf, "choice", "commit-glove-target", revision, "revision")
        require(save.read_bytes() == committed_bytes and committed_bytes != pending,
                "A stale activation changed the save")
        refused(ny, "choice", "commit-glove-target", committed["revision"], "unavailable")

    with (evidence / "manager-reopen.jsonl").open("w") as log, manager(
        executable, save, assets, evidence, log, "sf", args=args,
    ) as reopened:
        result = observe(reopened)
        require(result["observation"] == final, "Reopen lost the committed field")
        refused(reopened, "choice", "commit-glove-target", result["revision"], "unavailable")
        require(save.read_bytes() == committed_bytes, "Repeated activation changed the save")
    recovered = evidence / "recovered.json"
    with (evidence / "manager-recover.jsonl").open("w") as log, manager(
        executable, recovered, assets, evidence, log, "sf", args=(*args, "--recover-from", str(save)),
    ) as recovery:
        restored = observe(recovery)
        require(restored["observation"] == view and restored["revision"] == revision
                and control(restored, "choice", "commit-glove-target") is None,
                "Recovery must reopen the same pending choice with previews cleared")
    require(save.read_bytes() == committed_bytes,
            "Recovery changed the original committed save")
    return committed_bytes


@contextmanager
def desktop_window(executable, assets, evidence, save, seat, practice, size, stage):
    from native_market import client_bounds

    title = TITLE if practice else EXPERT_TITLE
    args = ("--review-glove",) if practice else ()
    with launch(executable, save, assets, evidence / f"{stage}-{seat}.log", seat,
                title=title, args=args, origin=(0, 0)) as (game, window, _):
        xdotool("windowsize", "--sync", window, *size)
        bounds = (0, 0, *size)
        wait_for(lambda: client_bounds(window) == bounds, f"an actual {size[0]} × {size[1]} X11 client")
        focus(window)
        yield game, window, bounds


def review_desktop(executable, assets, evidence, practice, box, size, pending, committed):
    save = evidence / "desktop.json"
    save.write_bytes(pending)
    with desktop_window(executable, assets, evidence, save, "ny", practice, size, "pending") as (ny, window, bounds):
        capture_review(bounds, evidence / "ny-pending.png", no_choices)
        click(window, choice_point(size, box, commit=True))
        require(save.read_bytes() == pending, "NY's native window accepted an SF decision")
        close(ny, window)
    with desktop_window(executable, assets, evidence, save, "sf", practice, size, "pending") as (sf, window, bounds):
        opening = capture_review(bounds, evidence / "sf-pending.png", lambda image: selection_is(image, None))
        require(pawn_at(opening, 2) == practice, "The initial second-base runner is incorrect")
        xdotool("key", "Tab")
        capture_review(bounds, evidence / "sf-batter-help.png", lambda image: (
            border_at(image, 1240, 281, (FOCUS,))
            and image.getpixel(design_point(size, 690, 470)) == HELP
        ))
        xdotool("key", "Escape")
        capture_review(bounds, evidence / "sf-batter-dismissed.png",
                       lambda image: image.getpixel(design_point(size, 690, 470)) != HELP)
        click(window, design_point(size, 1597, 426))
        capture_review(bounds, evidence / "sf-defender-help.png",
                       lambda image: image.getpixel(design_point(size, 690, 470)) == HELP)
        focus(window)
        for step, selected in enumerate((box, 3 - box, box), 1):
            click(window, choice_point(size, selected))
            image = capture_review(bounds, evidence / f"sf-preview-{step}.png",
                                   lambda image: selection_is(image, selected))
            live = design_rect(size, 0, 0, 1070, 965)
            require(not ImageChops.difference(opening.crop(live), image.crop(live)).getbbox(),
                    "A native preview changed the stadium, score or runners")
            require(save.read_bytes() == pending, "A native preview wrote to the committed save")
        close(sf, window)

    with desktop_window(executable, assets, evidence, save, "sf", practice, size, "reopen") as (sf, window, bounds):
        capture_review(bounds, evidence / "sf-reopened-pending.png", lambda image: selection_is(image, None))
        require(save.read_bytes() == pending, "The pending Glove changed after reopening")
        for target in range(2, box - 1, -1):
            xdotool("key", "shift+Tab")
            capture_review(bounds, evidence / f"sf-keyboard-box-{target}.png", lambda image: (
                border_at(image, 1310 + (target - 1) * 392, 614, (FOCUS,))
            ))
        xdotool("key", "Return")
        capture_review(bounds, evidence / "sf-selected.png", lambda image: selection_is(image, box))
        require(save.read_bytes() == pending, "Keyboard preview committed Glove")
        xdotool("key", "Tab")
        next_focus_y = 614 if box == 1 else 875
        capture_review(bounds, evidence / "sf-selected-with-focus-moved.png", lambda image: (
            selection_is(image, box)
            and border_at(image, 1702, next_focus_y, (FOCUS,))
        ))
        require(save.read_bytes() == pending, "Traveling away from the selection changed Glove")
        click(window, choice_point(size, box, commit=True), button=3)
        require(save.read_bytes() == pending, "Secondary click committed Glove")
        click(window, choice_point(size, box, commit=True))
        wait_for(lambda: save.read_bytes() != pending, "the primary Use Glove action")
        require(save.read_bytes() == committed, "Manager-seat and native commits differ")
        click(window, choice_point(size, box, commit=True))
        require(save.read_bytes() == committed, "A repeated native click changed the save")

        def resolved(image):
            return (stadium_rendered(image) and no_choices(image) and not pawn_at(image, 2)
                    and pawn_at(image, 1) == (box == 2))

        after = capture_review(bounds, evidence / "sf-resolved.png", resolved)
        park = design_rect(size, 39, 248, 1067, 530)
        require(not ImageChops.difference(opening.crop(park), after.crop(park)).getbbox(),
                "The native stadium moved while resolving the Glove")
        close(sf, window)
    with desktop_window(executable, assets, evidence, save, "ny", practice, size, "resolved") as (ny, window, bounds):
        observer = capture_review(bounds, evidence / "ny-resolved.png", resolved)
        stadium = design_rect(size, 36, 245, 1070, 848)
        require(not ImageChops.difference(after.crop(stadium), observer.crop(stadium)).getbbox(),
                "NY saw a different rendered stadium after the commitment")
        require(save.read_bytes() == committed, "Reopening the observing seat changed the save")
        close(ny, window)

    geometry = {
        "client": size,
        "stadium": design_rect(size, 36, 245, 1070, 848),
        "batter": design_point(size, 1251, 426),
        "defender": design_point(size, 1597, 426),
        "outcomes": [choice_point(size, target) for target in (1, 2)],
        "use_glove": [choice_point(size, target, commit=True) for target in (1, 2)],
    }
    (evidence / "geometry.json").write_text(json.dumps(geometry, indent=2) + "\n")


def main():
    from native_game_two import file_digest

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--assets", type=Path, help="Path to the audited owner-local collection")
    parser.add_argument("--binary", type=Path, help="Use an already built desktop executable")
    args = parser.parse_args()
    evidence = Path(tempfile.mkdtemp(prefix="baseball-native-glove."))
    print(f"Native Glove evidence in {evidence}", flush=True)
    try:
        check_display("scripts/native-glove", SIZES[1])
        executable = args.binary.resolve() if args.binary else build()
        (evidence / "inputs.sha256").write_text(f"{file_digest(executable)}  {executable}\n")
        for practice, name in ((True, "practice"), (False, "game-two")):
            fixture = evidence / name
            fixture.mkdir()
            pending = prepare_pending(executable, args.assets, fixture, practice)
            for box in (1, 2):
                branch = fixture / f"box-{box}"
                branch.mkdir()
                committed = check_branch(executable, args.assets, branch, practice, box, pending)
                for size in SIZES:
                    desktop = branch / f"{size[0]}x{size[1]}"
                    desktop.mkdir()
                    review_desktop(executable, args.assets, desktop, practice, box, size, pending, committed)
    except (OSError, subprocess.SubprocessError, RuntimeError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    print("Both Glove fixtures, both outcomes, both seats, reopen and recovery passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
