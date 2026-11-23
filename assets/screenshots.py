#!/usr/bin/env python3
"""Takes the screenshots in the README.

Run from the repository root on macOS, after `cargo build --release`:

    python3 assets/screenshots.py

It opens the same Ghostty window as assets/demo.py, with a fresh home folder,
and takes four screenshots: a Quotes practice halfway through, its results, the
lesson menu, and the practice menu in the light theme. Each is saved in
assets/images as a PNG and as a WebP, with the window's shadow. The PNG, for
the Snap Store, which doesn't take WebP, loses the shadow's faint outer rows to
fit the store's 2160 pixel height. It takes about 3
minutes, during which the Mac shouldn't be used. It needs what assets/demo.py
needs, and cwebp.
"""

import json
import re
import struct
import subprocess
import sys
import tempfile
import time
from pathlib import Path

# Importing demo would otherwise leave a __pycache__ folder in assets
sys.dont_write_bytecode = True
import demo  # noqa: E402

IMAGES = Path("assets/images")
WEBP_QUALITY = 85
SETTLE = 1
SNAP_STORE_HEIGHT = 2160

# An earlier Quotes session, slower than the one typed here, so the results
# show "New best!"
EARLIER_QUOTES = {
    "timestamp": "2026-11-01T20:00:00", "wpm": 42.0, "accuracy": 97.0,
    "correct": 580, "total": 598, "duration_secs": 170.0, "completed": True,
    "id": "quotes", "title": "Quotes", "layout": "qwerty",
}

# Letters typed wrong, as a neighbouring key, the first times each comes up:
# v, c, p and b are rare enough to end below 95% and show orange, and e is
# common enough to stay blue, so the weakest keys listed are four orange ones
# and a blue one
MISTAKES = {"v": ("c", 1), "c": ("x", 2), "p": ("o", 1), "b": ("n", 1), "e": ("r", 1)}

# The first screenshot is taken from this line on, just before a character
# typed with Shift, so the keyboard shows a Shift key as well as the next key
SHOT_LINE = 3
SHIFTED = set('~!@#$%^&*()_+{}|:"<>?')
QUOTES_IN_PRACTICE_MENU = 2
LESSON_IN_MENU = 2


def screen():
    return subprocess.run(demo.TMUX + ["capture-pane", "-t", "demo", "-p"],
                          check=True, capture_output=True, text=True).stdout


def line_count():
    found = re.search(r"(\d+)/(\d+)", screen().splitlines()[0])
    if not found:
        raise SystemExit("The header shows no line count")
    return int(found.group(2))


def current_line():
    time.sleep(demo.MENU_STEP)
    # The text panel is the first box: a blank row, then the line being typed
    rows = [row for row in screen().splitlines() if "│" in row]
    line = rows[1].strip().strip("│").strip() if len(rows) > 1 else ""
    if not line:
        raise SystemExit("No line to type was found on the screen")
    return line


def with_mistakes(line, made):
    typed = []
    for ch in line:
        wrong, times = MISTAKES.get(ch, (ch, 0))
        if made.get(ch, 0) < times:
            made[ch] = made.get(ch, 0) + 1
            typed.append(wrong)
        else:
            typed.append(ch)
    return "".join(typed)


def shot(window_id, name):
    time.sleep(SETTLE)
    png = IMAGES / f"{name}.png"
    subprocess.run(["screencapture", "-x", f"-l{window_id}", str(png)], check=True)
    subprocess.run(["cwebp", "-quiet", "-q", str(WEBP_QUALITY), str(png),
                    "-o", str(png.with_suffix(".webp"))], check=True)
    fit_snap_store(png)
    print(f"Took {png}")


def fit_snap_store(png):
    """Trims the shadow's outer rows, which are almost transparent, top and
    bottom, until the PNG is no higher than the Snap Store allows"""
    width, height = struct.unpack(">II", png.read_bytes()[16:24])
    if height > SNAP_STORE_HEIGHT:
        subprocess.run(["sips", "--cropToHeightWidth", str(SNAP_STORE_HEIGHT), str(width), str(png)],
                       check=True, capture_output=True)


def press(key, times=1):
    for _ in range(times):
        demo.press(key)
        time.sleep(demo.MENU_STEP)


def main():
    if not demo.APP.exists():
        raise SystemExit("Run `cargo build --release` first")

    home = Path(tempfile.mkdtemp(prefix="clavirio-screenshots-"))
    data = home / ".clavirio"
    data.mkdir()
    (data / "history.json").write_text(json.dumps([EARLIER_QUOTES]))

    try:
        _, window_id = demo.open_window(home)

        press("Down", LESSON_IN_MENU)
        shot(window_id, "screenshot_dark_3")

        press("m")
        press("Down", QUOTES_IN_PRACTICE_MENU)
        press("Enter")
        time.sleep(SETTLE)

        made = {}
        shot_taken = False
        for index in range(line_count()):
            typed = with_mistakes(current_line(), made)
            if index >= SHOT_LINE and not shot_taken:
                at = next((i for i, ch in enumerate(typed)
                           if i >= len(typed) // 4 and (ch.isupper() or ch in SHIFTED)), None)
                if at is not None:
                    demo.type_text(typed[:at])
                    shot(window_id, "screenshot_dark")
                    shot_taken = True
                    typed = typed[at:]
            demo.type_text(typed)
        if not shot_taken:
            raise SystemExit("No character typed with Shift came up for the first screenshot")
        shot(window_id, "screenshot_dark_2")

        press("Escape")
        press("t")
        press("Up", QUOTES_IN_PRACTICE_MENU)
        shot(window_id, "screenshot_light")
    finally:
        subprocess.run(demo.TMUX + ["kill-server"], capture_output=True)


if __name__ == "__main__":
    main()
