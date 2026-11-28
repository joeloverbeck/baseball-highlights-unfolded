"""Exercise the purchase-order decision in both manager seats and the Ubuntu window."""

import argparse
from contextlib import ExitStack
from pathlib import Path
import tempfile

from PIL import ImageChops

from native_buy_round import (
    build, capture_when, check_display, click, launch, position, require, wait_for, xdotool,
)
from native_game_two import FIXTURE, manager, observe


CONTRIBUTORS = (
    ("SF", 6, (("Rookie Cyborg", 2), ("Rookie Natural", 2), ("Rookie Natural", 1),
               ("Rookie Robot", 1), ("Veteran Natural", 0), ("Veteran Robot", 0))),
    ("NY", 7, (("Rookie Natural", 1), ("Veteran Natural", 1), ("Veteran Cyborg", 1),
               ("Veteran Robot", 2), ("Veteran Natural", 0), ("Veteran Robot", 2))),
)


def exercise_seats(executable, assets, evidence, first):
    save = evidence / "series.json"
    save.write_bytes(FIXTURE.read_bytes())
    start = save.read_bytes()
    with ExitStack() as exits:
        transcript = exits.enter_context((evidence / "manager-transcript.jsonl").open("w"))
        seats = [exits.enter_context(manager(executable, save, assets, evidence, transcript, seat))
                 for seat in ("sf", "ny")]
        for seat, request in enumerate(seats):
            initial = observe(request)
            view = initial["observation"]
            require(view["acting_manager"] == "san_francisco", "SF earned the order choice")
            require(view["decision"] == ({"decision": "choose_order"} if seat == 0 else None),
                    "Only the chooser may change the order")
            order_choices = {control["choice"] for control in initial["table"]["controls"]
                             if (control["choice"] or "").startswith("buy-first-")}
            require(order_choices == ({"buy-first-sf", "buy-first-ny"} if seat == 0 else set()),
                    "The chooser must see both actions, and the observer sees neither")
            for club, revenue, roster in zip(CONTRIBUTORS, view["revenue"], view["rosters"]):
                abbreviation, total, ledger = club
                require(revenue["total"] == total, f"Wrong earned total for {abbreviation}")
                ordered_controls = sorted(
                    (control for control in initial["table"]["controls"]
                     if (control["component"] or "").startswith(f"revenue-{abbreviation.lower()}-")),
                    key=lambda control: tuple(control["position"]),
                )
                require(len(ordered_controls) == len(ledger),
                        f"Expected all six contributors in the {abbreviation} ledger")
                for control, (name, amount), contribution, player in zip(
                    ordered_controls, ledger, revenue["contributions"], roster["in_play"],
                ):
                    require(control["on_screen"] and control["label"] == f"Inspect {abbreviation} {name} (+${amount})",
                            f"Incorrect {abbreviation} ledger order or entry for {name}")
                    require(contribution == {"card": player["card"], "amount": amount},
                            f"Incorrect public contribution for {name}")
                    request("activate", "activated", revision=initial["revision"],
                            component=control["component"])
                    inspected = observe(request)
                    require(inspected["table"]["revenue_inspection"] == player,
                            f"Inspection did not show the exact {abbreviation} {name} face")
                    require(inspected["observation"] == view and save.read_bytes() == start,
                            "Revenue inspection changed the save or pending choice")
            request("activate", "activated", revision=initial["revision"], component="close-inspection")
            require(observe(request)["table"]["revenue_inspection"] is None, "The face stayed open")
        chooser = seats[0]
        chooser("activate", "activated", revision=observe(chooser)["revision"], choice=f"buy-first-{first}")
        buyer = {"sf": "san_francisco", "ny": "new_york"}[first]
        for request in seats:
            view = observe(request)["observation"]
            require(view["first_buyer"] == view["buyer"] == buyer,
                    "Both seats must see the selected first buyer")
            require(view["events"][-1] == {"event": "order_chosen", "chooser": "san_francisco", "first": buyer},
                    "Both seats must see who chose the first buyer")


def point(size, x, y):
    return round(size[0] * x), round(size[1] * y)


def panel_visible(image):
    red, green, blue = image.getpixel(point(image.size, 0.978, 0.115))
    return red > 215 and green > 190 and 130 < blue < 220


def exercise_native(executable, assets, evidence, size, first):
    save = evidence / "native.json"
    save.write_bytes(FIXTURE.read_bytes())
    before = save.read_bytes()
    with launch(executable, save, assets, evidence / "native.log", "sf", origin=(0, 0)) as (_, window, _):
        xdotool("windowsize", "--sync", window, *size)
        table = (0, 0, *size)
        sf_first = point(size, 0.76, 0.80)
        ny_first = point(size, 0.92, 0.80)

        def order_ready(image):
            return all(image.getpixel(point(image.size, x, 0.78))[0] > 230 for x in (0.76, 0.92))

        capture_when(table, evidence / "order.png", order_ready)
        faces = []
        for index, y in enumerate((0.605, 0.657), start=1):
            click(window, point(size, 0.14, y))
            # Compare the face art at one location, away from the amounts and title.
            printed = (*point(size, 0.915, 0.20), *point(size, 0.96, 0.36))
            def expected_face(image):
                return panel_visible(image) and (
                    not faces or ImageChops.difference(faces[0], image.crop(printed)).getbbox() is not None
                )
            inspected = capture_when(table, evidence / f"rookie-natural-{index}.png", expected_face)
            faces.append(inspected.crop(printed))
            require(save.read_bytes() == before, "Native inspection changed the saved decision")
        click(window, sf_first if first == "sf" else ny_first)
        buyer = {"sf": "san_francisco", "ny": "new_york"}[first]
        wait_for(lambda: position(save)["first_buyer"] == buyer, "the selected order to be saved")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--assets", type=Path)
    parser.add_argument("--executable", type=Path)
    parser.add_argument("--seats-only", action="store_true")
    args = parser.parse_args()
    if not args.seats_only:
        check_display("scripts/native-buy-round --order", (3840, 2160))
    executable = args.executable.resolve() if args.executable else build()
    evidence = Path(tempfile.mkdtemp(prefix="baseball-purchase-order."))
    print(f"Evidence {evidence}", flush=True)
    for first, size in (("sf", (1080, 720)), ("ny", (3840, 2160))):
        run = evidence / first
        run.mkdir()
        exercise_seats(executable, args.assets, run, first)
        if not args.seats_only:
            exercise_native(executable, args.assets, run, size, first)
    print("Both manager seats accounted for all 12 contributions and saw either first buyer.")
    if not args.seats_only:
        print("Native X11 order inspection and selection passed at 1080 × 720 and 3840 × 2160.")


if __name__ == "__main__":
    main()
