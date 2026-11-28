"""Verify signing, demotion and the buying handoff through both manager interfaces."""

import argparse
from contextlib import ExitStack
from pathlib import Path
import shutil
import sys
import tempfile

from native_buy_round import build, require
from native_game_two import FIXTURE, activate, manager, observe


def control(reply, *, component=None, choice=None):
    return next((item for item in reply["table"]["controls"]
                 if item["component"] == component and item["choice"] == choice), None)


def review_managers(executable, assets, evidence):
    save = evidence / "manager.json"
    shutil.copyfile(FIXTURE, save)
    with ExitStack() as exits:
        transcript = exits.enter_context((evidence / "manager.jsonl").open("w"))
        sf = exits.enter_context(manager(executable, save, assets, evidence, transcript, "sf"))
        ny = exits.enter_context(manager(executable, save, assets, evidence, transcript, "ny"))

        activate(sf, "sf", [], save, "choice", "buy-first-sf")
        market = activate(sf, "sf", [], save, "component", "inspect-slot-5")
        foxx = market["offers"][4]["player"]["card"]
        activate(sf, "sf", [], save, "choice", f"sign-5-{foxx['instance']}")
        pending = observe(sf)
        eligible = pending["observation"]["decision"]["eligible"]
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
                and waiting["observation"]["balance"] == 2,
                "NY must see SF's active decision and funds while waiting")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--assets", type=Path)
    parser.add_argument("--executable", type=Path)
    args = parser.parse_args()
    evidence = Path(tempfile.mkdtemp(prefix="baseball-native-signing."))
    print(f"Signing evidence in {evidence}", flush=True)
    try:
        executable = args.executable.resolve() if args.executable else build()
        review_managers(executable, args.assets, evidence)
    except (OSError, RuntimeError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
