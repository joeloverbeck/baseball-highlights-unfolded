"""Verify Press Box offer reading through the manager seat and X11."""

import argparse
import os
import time
from pathlib import Path
import shutil
import sys
import tempfile

from PIL import ImageGrab

from native_buy_round import (
    build, check_display, click, close, launch, position, require, wait_for, xdotool,
)
from native_game_two import FIXTURE, activate, file_digest, manager, observe


def review_manager(executable, assets, evidence):
    save = evidence / "manager.json"
    shutil.copyfile(FIXTURE, save)
    with (evidence / "manager.jsonl").open("w") as transcript, manager(
        executable, save, assets, evidence, transcript, "sf",
    ) as request:
        activate(request, "sf", [], save, "choice", "buy-first-sf")
        initial = observe(request)
        before = save.read_bytes()
        revision = initial["revision"]

        def key(name):
            return request("key", "pressed", revision=revision, key=name)

        offers = sorted(
            (control for control in initial["table"]["controls"]
             if (control["component"] or "").startswith("inspect-slot-")),
            key=lambda control: control["component"],
        )
        require(len(offers) == 6 and all(offer["on_screen"] for offer in offers),
                "Each of the six offers must be reachable at rest")
        require(len({round(offer["position"][1]) for offer in offers}) == 1,
                "The six offers must stand on one baseline")
        require([offer["label"].split("$")[-1] for offer in offers] == ["7", "4", "6", "6", "4", "5"],
                "The Press Box row must show each price before inspection")

        first = key("Tab")
        require(first["table"]["focus"]["component"] == "inspect-slot-1",
                "Tab must focus the first face")
        reading = first["table"]["market_inspection"]
        require(reading["name"] == "Bucky Cano" and reading["budget"] == "$1 short",
                "The unaffordable $7 offer must explain the shortfall against $6")
        require([symbol["mark"] for symbol in reading["symbols"]] == ["A"],
                "Cano bears only the Average mark")
        require(all(not (control["choice"] or "").startswith("sign-")
                    for control in first["table"]["controls"]),
                "An unaffordable offer must have no signing action")

        dismissed = key("Escape")
        require(dismissed["table"]["market_inspection"] is None
                and dismissed["table"]["focus"]["component"] == "inspect-slot-1",
                "Escape must keep focus while closing help on the same face")
        require(observe(request)["table"]["market_inspection"] is None,
                "The same focused target reopened without a change")
        second = key("Tab")
        require(second["table"]["focus"]["component"] == "inspect-slot-2",
                "The next face should open again after Escape")
        help_shown = key("Tab")
        require(help_shown["table"]["focus"]["component"] == "offer-help-2",
                "The disclosure must be reachable from the face")
        sign_shown = key("Tab")
        require(sign_shown["table"]["focus"]["choice"].startswith("sign-2-"),
                "Tab must continue from the disclosure to the explicit Sign action")
        back = key("Shift+Tab")
        require(back["table"]["focus"]["component"] == "offer-help-2",
                "Reverse Tab must return from Sign to the same disclosure")

        request("point", "pointed", revision=revision, component="inspect-slot-5")
        foxx = observe(request)
        symbols = foxx["table"]["market_inspection"]["symbols"]
        require([symbol["mark"] for symbol in symbols] == ["S", "PH"],
                "Foxx bears Slow and PH marks")
        require("already on base" in symbols[0]["meaning"]
                and all(words in symbols[1]["meaning"] for words in ("hand", "On Deck", "unseen", "Lineup")),
                "Help must explain the actual runner and pinch-hitting rules")
        require(foxx["table"]["market_inspection"]["budget"] == "$6 − $4 = $2 left",
                "Foxx must show the exact balance arithmetic")
        require(foxx["revision"] == revision and foxx["observation"] == initial["observation"]
                and save.read_bytes() == before,
                "Pointer reading and keyboard travel must preserve the observation, revision and save")


# These colors identify visible chrome on the ordinary window. Card content,
# help and actions are found in the captured client rather than from seat pixels.
FRAME = (81, 107, 101)
FOCUS = (255, 242, 205)
GOLD = (245, 199, 122)
PAPER = (247, 232, 204)
SIZES = [(1080, 720), (3840, 2160)]


def row_runs(image, y, color, minimum, x_min=0, x_max=None):
    pixels = image.load()
    x_max = x_max or image.width
    beginning = None
    runs = []
    for x in range(x_min, x_max + 1):
        if x < x_max and pixels[x, y] == color:
            if beginning is None:
                beginning = x
        elif beginning is not None:
            if x - beginning >= minimum:
                runs.append((beginning, x))
            beginning = None
    return runs


def card_boxes(image):
    baselines = []
    for y in range(0, round(image.height * 0.52)):
        runs = row_runs(image, y, FRAME, image.width // 12)
        if len(runs) == 6:
            baselines.append((y, runs))
    if len(baselines) < 2:
        return None
    top, runs = baselines[0]
    bottom, _ = baselines[-1]
    if bottom - top < image.height / 5:
        return None
    return [(left, top, right, bottom) for left, right in runs]


def move(window, point):
    xdotool("mousemove", "--window", window, *point)


def client_bounds(window):
    geometry = dict(
        line.split("=", 1)
        for line in xdotool("getwindowgeometry", "--shell", window).stdout.splitlines()
    )
    x, y, width, height = (int(geometry[key]) for key in ("X", "Y", "WIDTH", "HEIGHT"))
    return x, y, x + width, y + height


def capture_client(bounds):
    return ImageGrab.grab(bbox=bounds, xdisplay=os.environ["DISPLAY"]).convert("RGB")


def annotation(image, card, scale, color, *, action=False):
    left, _, right, bottom = card
    start = bottom + round((51 if action else 27) * scale)
    end = bottom + round((90 if action else 56) * scale)
    x_min, x_max = (left, right) if action else (0, image.width)
    minimum = (right - left) // 2 if action else right - left
    for y in range(start, end):
        runs = row_runs(image, y, color, minimum, x_min, x_max)
        if runs:
            x1, x2 = runs[0]
            return (x1 + x2) // 2, y + round(8 * scale)
    return None


def review_window(executable, assets, evidence, save, size):
    width, height = size
    scale = min(width / 1080, height / 720)
    with launch(executable, save, assets, evidence / "desktop.log", seat="sf", origin=(0, 0)) as (
        game, window, _,
    ):
        xdotool("windowsize", "--sync", window, width, height)
        bounds = wait_for(
            lambda: client_bounds(window) if client_bounds(window) == (0, 0, width, height) else None,
            f"an actual {width} × {height} X11 client",
        )
        frame = lambda: capture_client(bounds)
        move(window, (width - 5, height - 5))

        # Wait for this view to render, then measure the two edges of its six complete faces.
        def rendered():
            image = frame()
            header = image.getpixel((10, 20))
            footer = image.getpixel((width - 10, height - 10))
            return image if header == (26, 41, 33) and footer in ((21, 40, 47), header) else None

        rest = wait_for(rendered, "the market to finish rendering")
        rest.save(evidence / "at-rest.png")
        boxes = card_boxes(rest)
        require(boxes and len(boxes) == 6, "Six framed faces must share one readable row")
        before = save.read_bytes()

        def point_at(card):
            left, top, right, bottom = card
            move(window, ((left + right) // 2, (top + bottom) // 2))

        def outline(card, color):
            left, top, right, _ = card
            return frame().getpixel(((left + right) // 2, top)) == color

        def help_at(card):
            return annotation(frame(), card, scale, PAPER)

        def sign_at(card):
            return annotation(frame(), card, scale, GOLD, action=True)

        # Each face leads down across a non-committing gap to its own help and action.
        for index, card in enumerate(boxes):
            left, _, right, bottom = card
            center = (left + right) // 2
            point_at(card)
            help_point = wait_for(
                lambda: help_at(card) if outline(card, GOLD) else None,
                f"the help beside offer {index + 1}",
            )
            require(not left <= help_point[0] <= right, "Help covered its source face or action")
            pause = (center, bottom + round(18 * scale))
            move(window, pause)
            time.sleep(0.4)
            require(help_at(card), f"Help closed in the gap beneath offer {index + 1}")
            click(window, pause)
            require(help_at(card) and save.read_bytes() == before,
                    f"The connecting gap committed or closed offer {index + 1}")
            move(window, (center, help_point[1]))
            time.sleep(0.35)
            move(window, help_point)
            time.sleep(0.4)
            require(help_at(card), f"Offer {index + 1} closed on travel into its help")
            action = sign_at(card)
            require(bool(action) == (index != 0),
                    "Only the unaffordable $7 offer must lack a Sign control")
            if action:
                move(window, (center, help_point[1]))
                time.sleep(0.35)
                move(window, action)
                time.sleep(0.4)
                require(help_at(card) and sign_at(card), "The Sign control lost its visible reading")
            frame().save(evidence / f"offer-{index + 1}.png")
            if index == 0:
                xdotool("key", "Escape")
                time.sleep(0.55)
                require(not help_at(card), "Escape reopened the aid under the stationary pointer")

        move(window, (width - 5, height - 5))
        time.sleep(0.35)
        xdotool("key", "Tab")
        first = boxes[0]

        def focus_on(card):
            return outline(card, FOCUS)

        wait_for(lambda: focus_on(first) and help_at(first), "Tab focus on the left-edge offer")
        xdotool("key", "Tab", "Tab")
        second = boxes[1]
        wait_for(lambda: focus_on(second) and help_at(second), "Tab travel to the second offer")
        xdotool("key", "Tab", "Tab")
        wait_for(lambda: annotation(frame(), second, scale, FOCUS, action=True),
                 "Tab focus on the actual Sign action")
        xdotool("key", "shift+Tab")
        wait_for(lambda: annotation(frame(), second, scale, FOCUS),
                 "reverse Tab focus on the same offer's help")
        xdotool("key", "Escape")
        wait_for(lambda: focus_on(second) and not help_at(second), "Escape returning to the source face")
        time.sleep(0.45)
        require(not help_at(second), "Escape reopened the help while the source face held focus")
        require(save.read_bytes() == before, "Native inspection or keyboard travel changed the save")

        foxx = boxes[4]
        point_at(foxx)
        action = wait_for(lambda: sign_at(foxx), "Foxx's explicit Sign action")
        click(window, action)
        wait_for(lambda: save.read_bytes() != before, "the explicit Foxx purchase")
        committed = save.read_bytes()
        click(window, action)
        require(save.read_bytes() == committed, "A stationary pointer could sign the replacement")
        require(position(save)["balances"] == [2, 7], "Foxx should leave SF with $2")
        frame().save(evidence / "foxx-signed.png")
        close(game, window)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--assets", type=Path, help="Path to the audited owner-local collection")
    parser.add_argument("--executable", type=Path, help="Use a specific built executable")
    args = parser.parse_args()
    evidence = Path(tempfile.mkdtemp(prefix="baseball-native-market."))
    print(f"Press Box evidence in {evidence}", flush=True)
    try:
        executable = args.executable.resolve() if args.executable else build()
        digest = file_digest(executable)
        check_display("scripts/native-buy-round --market", SIZES[-1])
        review_manager(executable, args.assets, evidence)
        for size in SIZES:
            native = evidence / f"{size[0]}x{size[1]}"
            native.mkdir()
            save = native / "series.json"
            shutil.copyfile(evidence / "manager.json", save)
            review_window(executable, args.assets, native, save, size)
        require(file_digest(executable) == digest, "The executable changed during the replay")
        (evidence / "inputs.sha256").write_text(
            f"{digest}  {executable}\n{file_digest(FIXTURE)}  {FIXTURE}\n")
    except (OSError, RuntimeError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    print("Verified Press Box inspection through the manager seat and both X11 client sizes.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
