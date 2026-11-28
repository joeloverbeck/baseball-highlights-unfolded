"""Exercise both Glove outcomes through the ordinary Ubuntu window."""

import argparse
import os
from pathlib import Path
from struct import iter_unpack
import subprocess
import sys
import tempfile

from PIL import ImageChops, ImageGrab

from native_buy_round import build, click, close, launch, position, require, wait_for, xdotool


TITLE = "Baseball Highlights / Glove Review"
BOXES = {1: (555, 440), 2: (1000, 440)}
BOX_COLOR_SAMPLES = {1: (400, 433), 2: (900, 433)}
COMMIT = (805, 580)
COMMIT_COLOR_SAMPLE = (400, 580)
PAPER = (235, 214, 176)
GOLD = (245, 199, 122)
CHOICE_PANEL = (247, 232, 204)
LIVE = (0, 0, 1280, 360)


def review_window(executable, save, assets, log, seat="sf"):
    return launch(executable, save, assets, log, seat, title=TITLE, args=("--review-glove",))


def focus(window):
    xdotool("windowraise", window, "windowfocus", "--sync", window)
    xdotool("mousemove", "--window", window, 5, 4)


def capture_when(table, path, ready):
    def frame():
        image = ImageGrab.grab(bbox=table, xdisplay=os.environ["DISPLAY"]).convert("RGB")
        return image if image.getpixel((1000, 760)) == PAPER and ready(image) else None

    image = wait_for(frame, f"the Glove review frame for {path.name}")
    image.save(path)
    return image


def selection_is(image, selected):
    return (
        image.getpixel((360, 418)) == CHOICE_PANEL
        and all(image.getpixel(point) == (GOLD if number == selected else PAPER)
            for number, point in BOX_COLOR_SAMPLES.items())
        and image.getpixel(COMMIT_COLOR_SAMPLE) == (GOLD if selected else PAPER)
    )


def observer_panel(image):
    return image.getpixel((360, 418)) == PAPER


def blue_pawn(image, point):
    x, y = point
    samples = iter_unpack("BBB", image.crop((x - 5, y - 5, x + 6, y + 6)).tobytes())
    return sum(blue > green + 35 and green > red + 35 for red, green, blue in samples) >= 30


def check_resolution(save, cancelled):
    payload = position(save)
    field = payload["field_review"]["field"]
    sf, ny = field["diamonds"]
    bases = [runner["card"]["identity"] if runner else None for runner in ny["bases"]]
    expected = (4, [None, None, None]) if cancelled == 1 else (3, ["FA-067", None, None])
    require((ny["score"], bases) == expected and sf["score"] == 1, "Wrong final field")
    require(not field["glove_pending"] and field["acting"] == "new_york", "Glove still pending")
    require(
        [(item["hit"], item["target"]["card"]["identity"]) for item in field["threatened"]]
        == [("single", "ST-001")],
        "Expected SF's single waiting at home",
    )
    cancellation, resolved_hit, placed = payload["events"]
    require(
        (cancellation["event"], cancellation["threat"]["target"]["number"],
         resolved_hit["event"], resolved_hit["threat"]["target"]["number"], placed["event"])
        == ("glove_cancelled", cancelled, "hit_resolved", 3 - cancelled, "threats_placed"),
        "Expected one cancellation, the surviving hit, then SF's single",
    )


def check_pending(executable, assets, evidence, save):
    with review_window(executable, save, assets, evidence / "sf-before.log") as (sf, window, table):
        focus(window)
        opening = capture_when(table, evidence / "sf-opening.png", lambda im: selection_is(im, None))
        require(blue_pawn(opening, (640, 227)), "Expected the average runner on second")
        pending = save.read_bytes()
        with review_window(executable, save, assets, evidence / "ny-before.log", "ny") as (ny, other, other_table):
            focus(other)
            observed = capture_when(other_table, evidence / "ny-opening.png", observer_panel)
            focus(window)
            for step, selected in enumerate((1, 2, 1), 1):
                click(window, BOXES[selected])
                focus(window)
                preview = capture_when(
                    table, evidence / f"sf-preview-{step}.png",
                    lambda im: selection_is(im, selected),
                )
                require(not ImageChops.difference(opening.crop(LIVE), preview.crop(LIVE)).getbbox(),
                        "A preview changed the live field or score")
                require(save.read_bytes() == pending, "A preview wrote to the committed save")
            focus(other)
            click(other, COMMIT)
            focus(other)
            observer_preview = capture_when(other_table, evidence / "ny-during-preview.png",
                                            observer_panel)
            require(not ImageChops.difference(observed, observer_preview).getbbox(),
                    "SF's local preview changed NY's window")
            require(save.read_bytes() == pending, "The observing seat wrote to the save")
            close(ny, other)
        close(sf, window)
    return pending


def commit(executable, assets, evidence, save, box, pending=None):
    with review_window(executable, save, assets, evidence / f"sf-commit-{box}.log") as (sf, window, table):
        focus(window)
        capture_when(table, evidence / f"sf-pending-{box}.png", lambda im: selection_is(im, None))
        if pending is not None:
            require(save.read_bytes() == pending, "Resumed a different pending Glove decision")
        click(window, BOXES[box])
        focus(window)
        capture_when(table, evidence / f"sf-selected-{box}.png", lambda im: selection_is(im, box))
        before = save.read_bytes()
        click(window, COMMIT, button=3)
        require(save.read_bytes() == before, "Secondary click committed the Glove")
        click(window, COMMIT)
        wait_for(lambda: save.read_bytes() != before, "a committed Glove")
        xdotool("key", "space")
        check_resolution(save, box)
        committed = save.read_bytes()
        click(window, COMMIT)
        require(save.read_bytes() == committed, "The same click committed the Glove twice")
        capture_when(table, evidence / f"sf-resolved-{box}.png",
                     lambda im: not blue_pawn(im, (640, 227))
                     and blue_pawn(im, (703, 275)) == (box == 2))
        close(sf, window)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--assets", type=Path, help="Path to the audited owner-local collection")
    args = parser.parse_args()
    evidence = Path(tempfile.mkdtemp(prefix="baseball-native-glove."))
    print(f"Native Glove evidence in {evidence}", flush=True)
    try:
        executable = build()
        first = evidence / "cancel-single.json"
        pending = check_pending(executable, args.assets, evidence, first)
        commit(executable, args.assets, evidence, first, 1, pending)
        commit(executable, args.assets, evidence, evidence / "cancel-home-run.json", 2)
    except (OSError, subprocess.SubprocessError, RuntimeError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    print("Native Glove preview switching, observation, restart and both commitments passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
