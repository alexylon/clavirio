#!/usr/bin/env python3
"""Records the animated demo, assets/images/demo.webp.

Run from the repository root on macOS, after `cargo build --release`:

    python3 assets/demo.py

It opens a Ghostty window on a tmux session, types QWERTY lesson 14 into
clavirio with a fresh home folder, records the window with screencapture and
makes the animated assets/images/demo.webp with ffmpeg and img2webp, leaving
out the middle of the lesson. The README and the website show demo.gif
instead, made by hand from the same frames, as Safari plays this WebP very
slowly.
It takes about 3 minutes, during which the Mac shouldn't be used: the window
must stay in front and on screen. It needs Ghostty, tmux, ffmpeg and img2webp,
and Screen Recording permission for the terminal that runs it.
"""

import json
import shutil
import subprocess
import tempfile
import time
from pathlib import Path

LESSON = Path("assets/lessons/qwerty/14_top_shift.txt")
LESSON_INDEX = 13
OUTPUT = Path("assets/images/demo.webp")
APP = Path("target/release/clavirio").resolve()

TMUX = [shutil.which("tmux") or "tmux", "-L", "clavirio-demo", "-f", "/dev/null"]
TITLE = "clavirio"
WINDOW_X, WINDOW_Y = 40, 40
COLUMNS, ROWS = 140, 46
WINDOW_WAIT = 10

SECONDS_PER_KEY = 0.2
MENU_STEP = 0.25
PAUSE = 1.5
RESULTS_SHOWN = 6
# Longer than the recording needs; screencapture finishes the file when it ends
RECORDING_LIMIT = 165

WIDTH = 1600
FPS = 20
QUALITY = 80

# Mistakes typed on purpose, in every line, as a learner makes them: mostly a
# neighbouring key, sometimes a capital typed without Shift. Q, W and P are
# missed more than once, so they end below 95% and show orange on the results
# keyboard; every other key missed is typed often enough to stay blue
MISTAKES = {
    0: [("Quiet", "quiet"), ("Poet", "Poer"), ("Youth", "Youtg")],
    1: [("Style", "Stule"), ("Daily", "Daoly"), ("Hard", "Hatd"), ("Work", "work")],
    2: [("Proud", "proud"), ("Great", "Grwat")],
    3: [("Usually", "Usuakly")],
    4: [("Quest", "Wuest")],
    5: [("Effort", "Effirt")],
    6: [("Worthy", "Eorthy"), ("Leadership", "Leadetship")],
    7: [("Types", "Tyoes")],
    8: [("Yearly", "Ywarly")],
    9: [("Paul", "paul")],
    10: [("Always", "Alwaya")],
    11: [("Group", "Grouo")],
    12: [("Quite", "Qyite")],
    13: [("Adapted", "Adspted")],
    14: [("Played", "played")],
}

FIND_WINDOW = """
ObjC.import("CoreGraphics");
const windows = ObjC.castRefToObject($.CGWindowListCopyWindowInfo($.kCGWindowListOptionOnScreenOnly, 0));
const found = [];
for (let i = 0; i < windows.count; i++) {
  const w = ObjC.deepUnwrap(windows.objectAtIndex(i));
  if (w.kCGWindowOwnerName === "Ghostty" && w.kCGWindowName === "%s") found.push([w.kCGWindowNumber, w.kCGWindowBounds, w.kCGWindowOwnerPID]);
}
JSON.stringify(found);
"""

# Brings Ghostty to the front, so the window has its coloured title bar buttons
ACTIVATE = """
ObjC.import("AppKit");
$.NSRunningApplication.runningApplicationWithProcessIdentifier(%d)
  .activateWithOptions($.NSApplicationActivateIgnoringOtherApps);
"""


def tmux(*args):
    subprocess.run(TMUX + list(args), check=True)


def press(key):
    tmux("send-keys", "-t", "demo", key)


def type_text(text):
    for ch in text:
        # tmux reads a lone ";" as the end of a command
        tmux("send-keys", "-t", "demo", "-l", "--", "\\;" if ch == ";" else ch)
        time.sleep(SECONDS_PER_KEY)


def lesson_lines():
    """The lesson's lines as typed, mistakes and all"""
    lines = LESSON.read_text().split("\n")
    for index, mistakes in MISTAKES.items():
        for right, wrong in mistakes:
            if right not in lines[index]:
                raise SystemExit(f"{LESSON} line {index + 1} no longer has {right!r}")
            lines[index] = lines[index].replace(right, wrong, 1)
    return lines


def open_window(home):
    """Opens clavirio in a Ghostty window in front, and returns the window's
    screen area and its id"""
    tmux("new-session", "-d", "-s", "demo", "-x", str(COLUMNS), "-y", str(ROWS),
         f"env HOME={home} {APP}")
    tmux("set", "-g", "status", "off")
    tmux("set", "-g", "default-terminal", "tmux-256color")
    tmux("set", "-as", "terminal-features", ",*:RGB")
    subprocess.run([
        "open", "-na", "Ghostty.app", "--args",
        f"--window-width={COLUMNS}", f"--window-height={ROWS}",
        f"--window-position-x={WINDOW_X}", f"--window-position-y={WINDOW_Y}",
        f"--title={TITLE}", "--window-save-state=never",
        # The padding takes the colour of the cells next to it, as the light
        # theme paints its own background
        "--window-padding-color=extend",
        "--quit-after-last-window-closed=true", "--confirm-close-surface=false",
        "--command=" + " ".join(TMUX[:3] + ["attach", "-t", "demo"]),
    ], check=True)

    for _ in range(WINDOW_WAIT):
        time.sleep(1)
        found = subprocess.run(["osascript", "-l", "JavaScript", "-e", FIND_WINDOW % TITLE],
                               check=True, capture_output=True, text=True).stdout
        # The window this script opened, not another Ghostty window with the same title
        windows = [w for w in json.loads(found) if w[1]["X"] == WINDOW_X]
        if windows:
            window_id, b, pid = windows[0]
            subprocess.run(["osascript", "-l", "JavaScript", "-e", ACTIVATE % pid], check=True,
                           capture_output=True)
            time.sleep(1)
            return f"{b['X']},{b['Y']},{b['Width']},{b['Height']}", window_id
    raise SystemExit("The Ghostty window wasn't found")


def record(region, movie):
    lines = lesson_lines()
    # The screen area of the window, which stays in front; recording the window
    # itself with -l leaves out its title bar buttons
    recording = subprocess.Popen(["screencapture", "-x", "-v", "-V", str(RECORDING_LIMIT),
                                  f"-R{region}", str(movie)])
    start = time.time()

    time.sleep(2 * PAUSE)
    for _ in range(LESSON_INDEX):
        press("Down")
        time.sleep(MENU_STEP)
    time.sleep(PAUSE / 2)
    press("Enter")
    time.sleep(PAUSE)

    type_text(lines[0])
    type_text(lines[1])
    shown_until = time.time() - start + PAUSE / 2
    time.sleep(PAUSE)

    for line in lines[2:]:
        type_text(line)
    results_from = time.time() - start + PAUSE / 3
    time.sleep(RESULTS_SHOWN + PAUSE)

    recording.wait()
    return shown_until, results_from


def make_webp(movie, shown_until, results_from):
    frames = movie.parent / "frames"
    frames.mkdir(exist_ok=True)
    cut = (
        f"[0:v]trim=0:{shown_until},setpts=PTS-STARTPTS[a];"
        f"[0:v]trim={results_from}:{results_from + RESULTS_SHOWN},setpts=PTS-STARTPTS[b];"
        f"[a][b]concat=n=2:v=1,fps={FPS},scale={WIDTH}:-1:flags=lanczos"
    )
    subprocess.run(["ffmpeg", "-loglevel", "error", "-y", "-i", str(movie),
                    "-filter_complex", cut, str(frames / "%05d.png")], check=True)
    subprocess.run(["img2webp", "-loop", "0", "-lossy", "-sharp_yuv", "-q", str(QUALITY),
                    "-d", str(round(1000 / FPS))] + [str(f) for f in sorted(frames.glob("*.png"))]
                   + ["-o", str(OUTPUT)], check=True, capture_output=True)


def main():
    if not APP.exists():
        raise SystemExit("Run `cargo build --release` first")

    tmp = Path(tempfile.mkdtemp(prefix="clavirio-demo-"))
    home = tmp / "home"
    home.mkdir()
    movie = tmp / "demo.mov"
    try:
        region, _ = open_window(home)
        shown_until, results_from = record(region, movie)
    finally:
        subprocess.run(TMUX + ["kill-server"], capture_output=True)

    make_webp(movie, shown_until, results_from)
    print(f"Made {OUTPUT} from {movie}")


if __name__ == "__main__":
    main()
