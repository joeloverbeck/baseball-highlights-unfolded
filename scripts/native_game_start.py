"""Inspect both public Dugouts and start Game 2 from the completed Expert Buy Round."""

import argparse
from contextlib import ExitStack
from pathlib import Path
import tempfile

from native_buy_round import (
    build, capture_when, check_display, click, close, launch, position, require, wait_for, xdotool,
)
from native_game_two import FIXTURE, activate, file_digest, manager, observe, purchase
from native_purchase_order import point


# Physical cards in the issue #8 fixture after the recorded demotions and signings.
DUGOUTS = (
    (("ST-002", 43), ("ST-005", 102), ("ST-008", 46), ("ST-011", 54), ("ST-014", 7)),
    (("ST-050", 107), ("ST-057", 110), ("ST-058", 47), ("ST-059", 58), ("ST-060", 81)),
)
HANDS = (
    (("FA-065", 28), ("ST-013", 98), ("ST-009", 104), ("ST-003", 91), ("ST-012", 14), ("ST-006", 19)),
    (("FA-067", 5), ("ST-054", 53), ("ST-049", 20), ("ST-048", 18), ("ST-046", 25), ("ST-052", 12)),
)
TEAMS = ("sf", "ny")


def card_keys(cards):
    return tuple((card["identity"], card["instance"]) for card in cards)


def named(reply, kind, name):
    return next(
        (control for control in reply["table"]["controls"]
         if control[kind] == name and control["on_screen"]),
        None,
    )


def inspect_seat(request, seat_index, save):
    before = observe(request)
    view = before["observation"]
    digest = file_digest(save)
    require(
        view.get("next_game") == {
            "game": 2, "home": "new_york", "first_player": "san_francisco", "wins": [0, 1],
        },
        "Both managers must see the upcoming host, first player, and series wins",
    )
    require(view["acting_manager"] == "san_francisco", "SF is entitled to start Game 2")
    start = named(before, "choice", "start-game-two")
    require(
        (start is None if seat_index else start and start["label"] == "Start Game 2"),
        "The sole start action must be labeled Start Game 2 and available only to SF",
    )

    def choose(name):
        request("activate", "activated", revision=before["revision"], component=name)
        read = observe(request)
        require(
            read["observation"] == view and read["revision"] == before["revision"]
            and file_digest(save) == digest,
            f"{name} changed the decision or save",
        )
        return read

    for team, roster, cards in zip(TEAMS, view["rosters"], DUGOUTS):
        require(
            roster["lineup"] == 10 and roster["active"] == 15 and roster["minor_leagues"] == 1,
            f"{team} must have ten in Lineup, five in Dugout and one outside its active roster",
        )
        require("hand" not in roster or not roster["hand"], "A hand appeared before the draw")
        require(card_keys(player["card"] for player in roster["dugout"]) == cards,
                f"{team} Dugout did not contain the exact five remaining public cards")
        opened = choose(f"open-dugout-{team}")
        inspection = opened["table"]["dugout_inspection"]
        require(inspection == {"seat": roster["seat"], "players": roster["dugout"], "player": None},
                f"Opening {team} did not reveal the correct public faces in order")
        for player in roster["dugout"]:
            key = f"inspect-{player['card']['identity']}"
            require(named(opened, "component", key), f"Missing the full-face control for {key}")
            inspected = choose(key)
            require(inspected["table"]["dugout_inspection"]["player"] == player,
                    f"{key} did not show the exact selected face")
            opened = choose("close-inspection")
        require(choose("close-dugout")["table"]["dugout_inspection"] is None,
                f"{team} Dugout stayed open")


def exercise_seats(executable, assets, evidence):
    save = evidence / "series.json"
    save.write_bytes(FIXTURE.read_bytes())
    trace = []
    with ExitStack() as exits:
        transcript = exits.enter_context((evidence / "manager-transcript.jsonl").open("w"))
        sf, ny = [exits.enter_context(manager(executable, save, assets, evidence, transcript, team))
                  for team in TEAMS]

        def choose_sf(kind, name):
            return activate(sf, "sf", trace, save, kind, name)

        def choose_ny(kind, name):
            return activate(ny, "ny", trace, save, kind, name)

        choose_sf("choice", "buy-first-sf")
        purchase(choose_sf, "FA-065", "ST-004", "FA-067")
        purchase(choose_ny, "FA-067", "ST-056", "FA-068")
        (evidence / "completed-buy-round.json").write_bytes(save.read_bytes())
        for index, request in enumerate((sf, ny)):
            inspect_seat(request, index, save)
        choose_sf("choice", "start-game-two")
        for index, request in enumerate((sf, ny)):
            reply = observe(request)
            view = reply["observation"]
            require(view["standing"] == {"game": 2, "home": "new_york", "wins": [0, 1]},
                    "Game 2 must enter the advertised matchup")
            require(view["acting_manager"] == "san_francisco", "SF must play first")
            require(card_keys(view["rosters"][index]["hand"]) == HANDS[index],
                    f"{TEAMS[index]} did not draw the exact modified opening hand")
            require("hand" not in view["rosters"][1-index], "The opponent's hand was disclosed")
            require(named(reply, "choice", "start-game-two") is None, "Game 2 can be started again")
            require(sum(event["event"] == "game_started" for event in view["events"]) == 1,
                    "Expected one committed game start")
    return file_digest(save)


def exercise_native(executable, assets, evidence, size, expected_digest):
    label = f"{size[0]}x{size[1]}"
    save = evidence / f"native-{label}.json"
    save.write_bytes((evidence / "completed-buy-round.json").read_bytes())
    before = file_digest(save)
    bounds = (0, 0, *size)
    for team in ("ny", "sf"):
        with launch(executable, save, assets, evidence / f"{label}-{team}.log",
                    seat=team, origin=(0, 0)) as (game, window, _):
            xdotool("windowsize", "--sync", window, *size)
            reported = xdotool("getwindowgeometry", "--shell", window).stdout
            (evidence / f"{label}-{team}-geometry.txt").write_text(reported)
            geometry = dict(line.split("=", 1) for line in reported.splitlines())
            require(tuple(int(geometry[key]) for key in ("WIDTH", "HEIGHT")) == size,
                    f"Client did not acquire {size} bounds")

            def capture(name, check):
                return capture_when(bounds, evidence / f"{label}-{team}-{name}.png", check)

            def club_panel(image):
                return image.getpixel(point(size, 0.04, 0.26)) == (36, 60, 64)

            def gold(image):
                red, green, blue = image.getpixel(point(size, 0.43, 0.68))
                return red > 200 and green > 150 and blue < 150

            def dugout_faces(image):
                return not club_panel(image) and all(
                    image.getpixel(point(size, x, 0.45)) not in ((21, 40, 47), (36, 60, 64), (64, 94, 99))
                    for x in (0.134, 0.317, 0.5, 0.683, 0.866)
                )

            def enlarged_face(image):
                return not dugout_faces(image) and image.getpixel(point(size, 0.13, 0.45)) == (21, 40, 47) and (
                    image.getpixel(point(size, 0.50, 0.43)) != (21, 40, 47)
                )

            capture("matchup", lambda image: club_panel(image) and gold(image) == (team == "sf"))
            for pile, x in (("sf", 0.19), ("ny", 0.81)):
                click(window, point(size, x, 0.72))
                capture(f"{pile}-dugout", dugout_faces)
                # Open the final face from each five-card row, then return to the matchup.
                click(window, point(size, 0.86, 0.47))
                capture(f"{pile}-face", enlarged_face)
                require(file_digest(save) == before, "Native public inspection changed the save")
                click(window, point(size, 0.50, 0.87))
                capture(f"{pile}-returned", dugout_faces)
                click(window, point(size, 0.50, 0.87))
                capture(f"{pile}-closed", club_panel)

            click(window, point(size, 0.50, 0.68))
            if team == "ny":
                capture("still-waiting", club_panel)
                require(file_digest(save) == before, "NY activated SF's Start Game 2")
            else:
                wait_for(lambda: file_digest(save) == expected_digest, "the same Game 2 as the manager replay")
                click(window, point(size, 0.50, 0.68))
                capture("started", lambda image: not club_panel(image))
                require(file_digest(save) == expected_digest, "Starting twice changed the committed save")
                require(position(save)["opening"] is not None, "The native window didn't enter Game 2")
            close(game, window)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--assets", type=Path)
    parser.add_argument("--executable", type=Path)
    parser.add_argument("--seats-only", action="store_true")
    args = parser.parse_args()
    if not args.seats_only:
        check_display("scripts/native-buy-round --start-game", (3840, 2160))
    executable = args.executable.resolve() if args.executable else build()
    evidence = Path(tempfile.mkdtemp(prefix="baseball-game-start."))
    print(f"Evidence {evidence}", flush=True)
    (evidence / "executable.sha256").write_text(f"{file_digest(executable)}  {executable}\n")
    (evidence / "fixture.sha256").write_text(f"{file_digest(FIXTURE)}  {FIXTURE}\n")
    started = exercise_seats(executable, args.assets, evidence)
    print("Both manager seats inspected both public Dugouts. SF started once with the expected private hands.")
    if not args.seats_only:
        for size in ((1080, 720), (3840, 2160)):
            exercise_native(executable, args.assets, evidence, size, started)
        print("Native X11 Dugout inspection and Game 2 start passed at 1080 × 720 and 3840 × 2160.")


if __name__ == "__main__":
    main()
