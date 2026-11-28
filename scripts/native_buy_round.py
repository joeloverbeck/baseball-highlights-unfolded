"""Verify the reviewed Buy Round in the ordinary Ubuntu window."""

import argparse
import ctypes as x11
import json
import os
from pathlib import Path
import re
from struct import iter_unpack
import subprocess
import sys
import tempfile
import time
from contextlib import contextmanager

from PIL import ImageGrab


TITLE = "Baseball Highlights / Expert Buy Round"
GAME_SIZE = (1280, 800)
# Coordinates in the ordinary launch at 96 DPI. A layout change may move them.
SF_FIRST = (91, 287)
RIZZUTO_OFFER = (493, 382)
SLOT_FIVE = (493, 507)
SIGN_FOXX = (1124, 659)
SIGN_MARIS = (1124, 642)
SECOND_IN_PLAY = (253, 689)
CONFIRM_DEMOTION = (1124, 614)
FINISH_BUYING = (195, 259)
ROUND_HISTORY = (380, 286)
PILE_CENTERS = {"Free Agent Deck": 800, "Lineup": 925, "Minor Leagues": 1176}


def build():
    command = [
        "cargo", "build", "--locked", "--bin", "baseball-highlights-unfolded",
        "--message-format=json-render-diagnostics",
    ]
    executable = None
    with subprocess.Popen(command, stdout=subprocess.PIPE, text=True) as cargo:
        for line in cargo.stdout:
            message = json.loads(line)
            if message["reason"] == "compiler-message":
                diagnostic = message["message"]["rendered"]
                if diagnostic:
                    print(diagnostic, file=sys.stderr, end="")
            if (message["reason"] == "compiler-artifact"
                    and message["target"]["name"] == "baseball-highlights-unfolded"):
                executable = message["executable"]
        if cargo.wait() or executable is None:
            raise RuntimeError("Cargo failed to build the ordinary Ubuntu game")
    return executable


def wait_for(check, description):
    deadline = time.monotonic() + 12
    while time.monotonic() < deadline:
        result = check()
        if result:
            return result
        time.sleep(0.15)
    raise RuntimeError(f"Timed out waiting for {description}")


def vulkan_adapter(log):
    output = log.read_text()
    adapter = re.search(r'AdapterInfo \{ name: "([^"]+)".*backend: Vulkan', output)
    if not adapter:
        raise RuntimeError(f"Game did not report a Vulkan adapter. See {log}")
    name = adapter[1]
    requested = os.environ.get("WGPU_ADAPTER_NAME")
    if requested and requested.casefold() not in name.casefold():
        raise RuntimeError(f"Requested {requested}, game used {name}. See {log}")
    return name


def xdotool(*args, check=True):
    return subprocess.run(
        ["xdotool", *map(str, args)], capture_output=True, text=True, check=check, timeout=5,
    )


def check_display(launcher, minimum=GAME_SIZE):
    display = os.environ.get("DISPLAY")
    if not display:
        raise RuntimeError(f"Launch through {launcher} to start the isolated display.")
    try:
        width, height = map(int, xdotool("getdisplaygeometry").stdout.split())
        ImageGrab.grab(bbox=(0, 0, 1, 1), xdisplay=display)
    except (OSError, subprocess.SubprocessError, RuntimeError, ValueError) as error:
        raise RuntimeError(f"Could not capture the X11 display for {launcher} ({display}): {error}") from error
    min_width, min_height = minimum
    if width < min_width or height < min_height:
        raise RuntimeError(
            f"{launcher} requires a display of at least {min_width}x{min_height}, "
            f"found {width}x{height} on {display}."
        )


@contextmanager
def launch(executable, save, assets, log, seat=None, *, title=TITLE, args=(), origin=None):
    command = [executable, "--save", str(save), *args]
    if seat:
        command += ["--seat", seat]
    if assets:
        command += ["--assets", str(assets)]
    env = os.environ.copy()
    env.pop("WAYLAND_DISPLAY", None)
    env["RUST_LOG"] = "info"
    with log.open("w") as output, subprocess.Popen(
        command, env=env, stdout=output, stderr=output,
    ) as game:
        def find_window():
            if game.poll() is not None:
                raise RuntimeError(f"Game exited with status {game.returncode}. See {log}")
            match = xdotool(
                "search", "--onlyvisible", "--all", "--pid", game.pid,
                "--name", f"^{title}$", check=False,
            )
            return match.stdout.strip() if match.returncode == 0 else None

        try:
            window = wait_for(find_window, f"the game window. See {log}")
            print(f"Vulkan adapter ({log.stem}) {vulkan_adapter(log)}", flush=True)
            if origin is not None:
                xdotool("windowmove", "--sync", window, *origin)
            xdotool("windowraise", window, "windowfocus", "--sync", window)
            geometry = dict(
                line.split("=", 1)
                for line in xdotool("getwindowgeometry", "--shell", window).stdout.splitlines()
            )
            x, y, width, height = (int(geometry[key]) for key in ("X", "Y", "WIDTH", "HEIGHT"))
            if (width, height) != GAME_SIZE:
                raise RuntimeError(f"Expected the default 1280x800 window, found {width}x{height}")
            if origin is not None and (x, y) != origin:
                raise RuntimeError(f"Expected window {window} at {origin}, found {(x, y)}. See {log}")
            yield game, window, (x, y, x + width, y + height)
        finally:
            if game.poll() is None:
                game.terminate()
                try:
                    game.wait(timeout=4)
                except subprocess.TimeoutExpired:
                    game.kill()
                    game.wait()


def click(window, point, button=1):
    xdotool(
        "mousemove", "--window", window, *point, "sleep", "0.15",
        "mousedown", button, "sleep", "0.2", "mouseup", button,
    )
    time.sleep(0.35)


class ClientMessage(x11.Structure):
    _fields_ = [
        ("type", x11.c_int), ("serial", x11.c_ulong), ("send_event", x11.c_int),
        ("display", x11.c_void_p), ("window", x11.c_ulong), ("atom", x11.c_ulong),
        ("format", x11.c_int), ("data", x11.c_long * 5),
    ]


class Event(x11.Union):
    _fields_ = [("client", ClientMessage), ("padding", x11.c_long * 24)]


def close(game, window):
    """Send the desktop close message to the verified window on this display."""
    native = x11.CDLL("libX11.so.6")
    native.XOpenDisplay.argtypes = [x11.c_char_p]
    native.XOpenDisplay.restype = x11.c_void_p
    native.XInternAtom.argtypes = [x11.c_void_p, x11.c_char_p, x11.c_int]
    native.XInternAtom.restype = x11.c_ulong
    native.XSendEvent.argtypes = [
        x11.c_void_p, x11.c_ulong, x11.c_int, x11.c_long, x11.POINTER(Event),
    ]
    native.XCloseDisplay.argtypes = [x11.c_void_p]
    display = native.XOpenDisplay(os.environ["DISPLAY"].encode())
    if not display:
        raise RuntimeError("Could not connect to the isolated X11 display")
    try:
        protocols = native.XInternAtom(display, b"WM_PROTOCOLS", 0)
        delete = native.XInternAtom(display, b"WM_DELETE_WINDOW", 0)
        message = ClientMessage(
            33, 0, 1, display, int(window), protocols, 32, (x11.c_long * 5)(delete),
        )
        if not native.XSendEvent(display, int(window), 0, 0, x11.byref(Event(client=message))):
            raise RuntimeError("Could not send the window close message")
    finally:
        native.XCloseDisplay(display)
    if game.wait(timeout=8):
        raise RuntimeError("Game did not exit successfully after the window close")


def capture_when(table, path, ready):
    latest = None

    def frame():
        nonlocal latest
        latest = ImageGrab.grab(bbox=table, xdisplay=os.environ["DISPLAY"]).convert("RGB")
        return latest if ready(latest) else None

    try:
        return wait_for(frame, f"the expected frame. See {path}")
    finally:
        if latest is not None:
            latest.save(path)


def capture(table, path, *, piles=None):
    def matches_piles(image):
        if piles is None:
            return True

        # Sample dark brown print inside each back. The exposed field and its
        # markings contain very little of this color at these three positions.
        for name, center in PILE_CENTERS.items():
            crop = image.crop((center - 12, 99, center + 12, 136))
            dark_ink = sum(
                red < 100 and red > green + 8 and green >= blue
                for red, green, blue in iter_unpack("BBB", crop.tobytes())
            )
            if (dark_ink >= 60) != (name in piles):
                return False
        return True

    capture_when(table, path, matches_piles)


def record(save):
    return json.loads(save.read_bytes())["payload"]


def position(save):
    return record(save)["position"]


def require(condition, description):
    if not condition:
        raise RuntimeError(description)


def smoke(executable, assets, evidence):
    save = evidence / "series.json"
    piles = {"Free Agent Deck", "Lineup"}
    with launch(executable, save, assets, evidence / "first.log") as (game, window, table):
        series = record(save)["series"]
        before = position(save)
        require(before["phase"] == "Order" and before["balances"] == [6, 7],
                "Expected an order decision with SF $6 and NY $7")
        capture(table, evidence / "opening.png", piles=piles)
        click(window, SF_FIRST)
        wait_for(lambda: position(save)["buyer"] == "san_francisco", "SF to buy first")
        click(window, RIZZUTO_OFFER)
        capture(table, evidence / "rizzuto.png")
        click(window, SLOT_FIVE)
        capture(table, evidence / "foxx.png")
        initial_bytes = save.read_bytes()
        click(window, SIGN_FOXX, button=3)
        require(save.read_bytes() == initial_bytes, "Secondary click changed the saved series")
        click(window, SIGN_FOXX)
        wait_for(lambda: position(save)["phase"] == "Demotion", "Foxx to sign")
        after = position(save)
        require(after["balances"] == [2, 7] and after["row"][4]["identity"] == "FA-067",
                "Expected $2 and Mickey Maris in slot 5")
        require(after["clubs"][0]["lineup"][-1]["identity"] == "FA-065", "Foxx did not top SF's Lineup")
        capture(table, evidence / "refill.png", piles=piles)
        pending_bytes = save.read_bytes()
        close(game, window)

    with launch(executable, save, assets, evidence / "resumed.log") as (game, window, table):
        require(save.read_bytes() == pending_bytes, "The resumed save differs from the pending demotion")
        capture(table, evidence / "resumed.png", piles=piles)
        click(window, SECOND_IN_PLAY)
        capture(table, evidence / "demotion.png")
        click(window, CONFIRM_DEMOTION)
        wait_for(lambda: position(save)["phase"] == "Shopping", "ST-004 to enter the Minor Leagues")
        demoted = position(save)
        sf = demoted["clubs"][0]
        require(record(save)["series"] == series, "Resumed a different series")
        require(sf["minor_leagues"][-1]["identity"] == "ST-004", "The selected card was not demoted")
        require(len(sf["lineup"]) + len(sf["in_play"]) == 15, "SF did not return to 15 active players")
        capture(table, evidence / "sf-demoted.png", piles=piles | {"Minor Leagues"})
        click(window, FINISH_BUYING)
        wait_for(lambda: position(save)["buyer"] == "new_york", "NY to take over")
        close(game, window)

    with launch(executable, save, assets, evidence / "ny.log", seat="ny") as (game, window, table):
        capture(table, evidence / "ny-opening.png", piles=piles)
        click(window, SLOT_FIVE)
        capture(table, evidence / "maris.png")
        click(window, SIGN_MARIS)
        wait_for(lambda: position(save)["phase"] == "Demotion", "Maris to sign")
        signed = position(save)
        require(signed["balances"] == [0, 0] and signed["row"][4]["identity"] == "FA-068",
                "NY must spend $7 and reveal Babe Bench in slot 5")
        require(signed["clubs"][1]["lineup"][-1]["identity"] == "FA-067",
                "Maris did not top NY's Lineup")
        click(window, SECOND_IN_PLAY)
        click(window, CONFIRM_DEMOTION)
        wait_for(lambda: position(save)["phase"] == "Shopping", "NY to finish its demotion")
        click(window, FINISH_BUYING)
        wait_for(lambda: position(save)["phase"] == "Complete", "both managers to finish")
        complete = position(save)
        require(complete["balances"] == [0, 0], "Revenue carried forward")
        for club, expected in zip(complete["clubs"], ("ST-004", "ST-056")):
            require(not club["in_play"], "In Play cards remained after cleanup")
            require(len(club["dugout"]) == 5 and len(club["lineup"]) == 10,
                    "Expected five cards in the Dugout and ten in the Lineup")
            require([card["identity"] for card in club["minor_leagues"]] == [expected],
                    f"Expected only {expected} in the Minor Leagues")
        capture(table, evidence / "finished.png", piles=piles | {"Minor Leagues"})
        click(window, ROUND_HISTORY)
        capture(table, evidence / "history.png")
        close(game, window)


def main():
    parser = argparse.ArgumentParser(
        description=__doc__,
        epilog="Use scripts/native-buy-round --game-two to replay the opening on both interfaces.",
    )
    parser.add_argument("--assets", type=Path, help="Path to the audited owner-local collection")
    args = parser.parse_args()
    evidence = Path(tempfile.mkdtemp(prefix="baseball-native-buy-round."))
    print(f"Native Buy Round evidence in {evidence}", flush=True)
    try:
        check_display("scripts/native-buy-round")
        smoke(build(), args.assets, evidence)
    except (OSError, subprocess.SubprocessError, RuntimeError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    print("Native order, both signings and refills, restart, demotions and Dugout cleanup passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
