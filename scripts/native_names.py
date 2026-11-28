"""Choose an exact same-name starter through the manager seat and Ubuntu window."""

import argparse
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from native_buy_round import (
    GAME_SIZE, build, capture, capture_when, check_display, click, close, launch, require, wait_for,
)
from native_game_two import FIXTURE, activate, file_digest, manager, observe
from native_glove import focus


ROOKIES = (
    {"identity": "ST-004", "instance": 10},
    {"identity": "ST-005", "instance": 102},
)
COLLECTOR_NUMBER = re.compile(r"\b(?:ST|FA)-\d{3}\b")


def named_label(reply, name, player):
    control = next(
        (item for item in reply["table"]["controls"] if name in (item["component"], item["choice"])),
        None,
    )
    require(control and control["on_screen"], f"The named control {name} is off screen")
    label = control["label"]
    require(player in label, f"Expected {player} in the control label, found {label!r}")
    require(not COLLECTOR_NUMBER.search(label), f"Collector number in the control label {label!r}")
    return control


def choose_starter(executable, assets, evidence, card):
    agent_save = evidence / "agent.json"
    desktop_save = evidence / "desktop.json"
    shutil.copyfile(FIXTURE, agent_save)
    shutil.copyfile(FIXTURE, desktop_save)
    trace = []
    transcript_path = evidence / "manager.jsonl"
    with transcript_path.open("w") as transcript, manager(
        executable, agent_save, assets, evidence, transcript, "sf",
    ) as request:
        def choose(kind, name):
            return activate(request, "sf", trace, agent_save, kind, name)

        choose("choice", "buy-first-sf")
        named_label(observe(request), "inspect-slot-5", "Frankie Foxx")
        market = choose("component", "inspect-slot-5")
        sign = f"sign-5-{market['offers'][4]['player']['card']['instance']}"
        named_label(observe(request), sign, "Frankie Foxx")
        pending = choose("choice", sign)
        both = [player["card"] for player in pending["rosters"][0]["in_play"]
                if player["name"] == "Rookie Natural"]
        require(both == list(ROOKIES), "Expected the two distinct Rookie Natural starters")
        visible = observe(request)
        first, second = [named_label(visible, f"inspect-{rookie['identity']}", "Rookie Natural")
                         for rookie in ROOKIES]
        require(first["position"] != second["position"], "The two starters share one inspect location")
        before = agent_save.read_bytes()
        inspection = choose("component", f"inspect-{card['identity']}")
        require(inspection == pending and agent_save.read_bytes() == before,
                "Inspecting a starter changed the manager decision or save")
        demote = f"demote-{card['identity']}-{card['instance']}"
        named_label(observe(request), demote, "Rookie Natural")
        committed = choose("choice", demote)
        require(committed["events"][-1] == {"event": "demoted", "seat": "san_francisco", "card": card},
                "The selected Rookie Natural was not the exact starter demoted")

    with launch(executable, desktop_save, assets, evidence / "desktop.log", seat="sf", origin=(0, 0)) as (
        game, window, bounds,
    ):
        capture_when(bounds, evidence / "opening.png", lambda image: image.getpixel((10, 20)) == (26, 41, 33))
        for entry in trace:
            before = desktop_save.read_bytes()
            focus(window)
            click(window, entry.point)
            if entry.kind == "choice":
                wait_for(lambda: desktop_save.read_bytes() != before, entry.label)
            require(file_digest(desktop_save) == entry.save_digest,
                    f"Native action {entry.label} diverged from manager-seat replay")
            if entry.name == f"inspect-{card['identity']}":
                focus(window)
                capture(bounds, evidence / "selected.png")
        capture(bounds, evidence / "committed.png")
        close(game, window)

    before = desktop_save.read_bytes()
    with transcript_path.open("a") as transcript, manager(
        executable, desktop_save, assets, evidence, transcript, "sf",
    ) as request:
        require(observe(request)["observation"] == committed, "Reopened seat changed the demotion")
        require(desktop_save.read_bytes() == before, "Reopening the desktop save changed its bytes")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--assets", type=Path, help="Path to the audited owner-local collection")
    parser.add_argument("--executable", type=Path, help="Use a specific built executable")
    args = parser.parse_args()
    evidence = Path(tempfile.mkdtemp(prefix="baseball-native-names."))
    print(f"Native name evidence in {evidence}", flush=True)
    try:
        check_display("xvfb-run -a -s '-screen 0 1440x900x24 -dpi 96' python3 scripts/native_names.py", GAME_SIZE)
        executable = args.executable.resolve() if args.executable else build()
        digest = file_digest(executable)
        for card in ROOKIES:
            route = evidence / card["identity"]
            route.mkdir()
            choose_starter(executable, args.assets, route, card)
        require(file_digest(executable) == digest, "The executable changed during the replay")
        (evidence / "inputs.sha256").write_text(f"{digest}  {executable}\n{file_digest(FIXTURE)}  {FIXTURE}\n")
    except (OSError, subprocess.SubprocessError, RuntimeError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    print("Each Rookie Natural could be selected in the native window and reopened as the same physical instance.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
