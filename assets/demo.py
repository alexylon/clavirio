#!/usr/bin/env python3
"""Records the animated demo at the top of the README.

Run from the repository root on macOS, after `cargo build --release`:

    python3 assets/demo.py

It opens a Ghostty window on a tmux session, types QWERTY lesson 14 into
clavirio with a fresh home folder, records the window with screencapture and
makes the animated assets/images/demo.gif with ffmpeg and ImageMagick, leaving
out the middle of the lesson. It's a GIF because Safari plays an animated WebP
this large very slowly, and the Snap Store doesn't take WebP.
The recording takes about 3 minutes, during which the Mac shouldn't be used:
the window must stay in front and on screen. Making the GIF takes about 4 more.
It needs Ghostty, tmux, ffmpeg and ImageMagick, and Screen Recording permission
for the terminal that runs it.
"""

import json
import shutil
import subprocess
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

LESSON = Path("assets/lessons/qwerty/14_top_shift.txt")
LESSON_INDEX = 13
OUTPUT = Path("assets/images/demo.gif")
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
COLOURS = 256
FRAME_NAMES = "%05d.png"

# The recording's noise, which would change most of every GIF frame: a pixel
# that changes by less than this in each colour is kept as it was
NOISE = "3%"
# A screen that stays the same for this many frames or more is shown as its
# last frame throughout, as the recording gets sharper while the screen is still
STILL_FRAMES = 8
# The biggest change between two frames, colour by colour, as a black and white
# image of the pixels that changed for real
REAL_CHANGES = ["-compose", "difference", "-composite", "-separate",
                "-evaluate-sequence", "max", "-threshold", NOISE]

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


def ffmpeg(*args):
    subprocess.run(["ffmpeg", "-loglevel", "error", "-y", *args], check=True)


def cut(movie, shown_until, results_from, frames):
    """Saves the recording as frames, without the middle of the lesson"""
    keep = (
        f"[0:v]trim=0:{shown_until},setpts=PTS-STARTPTS[a];"
        f"[0:v]trim={results_from}:{results_from + RESULTS_SHOWN},setpts=PTS-STARTPTS[b];"
        f"[a][b]concat=n=2:v=1,fps={FPS},scale={WIDTH}:-1:flags=lanczos"
    )
    ffmpeg("-i", str(movie), "-filter_complex", keep, str(frames / FRAME_NAMES))


def changes(before, after):
    """Whether any pixel changes for real from one frame to the next"""
    result = subprocess.run(["magick", str(before), str(after), *REAL_CHANGES,
                             "-format", "%[fx:mean>0]", "info:"],
                            check=True, capture_output=True, text=True)
    return result.stdout.strip() == "1"


def hold(previous, frame, steady):
    """Saves the frame where it changes for real from the previous steady frame,
    and a little around, and the previous steady frame everywhere else"""
    subprocess.run(["magick", str(previous), str(frame),
                    "(", "-clone", "0", "-clone", "1", *REAL_CHANGES,
                    "-morphology", "Dilate", "Square:2", ")",
                    "-compose", "over", "-composite", f"PNG24:{steady}"], check=True)


def steady(frames, steadied):
    """Saves the frames without the recording's noise, so that each GIF frame
    holds only what changed on the screen"""
    paths = sorted(frames.glob("*.png"))
    with ThreadPoolExecutor() as pool:
        changed = [True] + list(pool.map(changes, paths, paths[1:]))

    start = 0
    while start < len(paths):
        end = start
        while end + 1 < len(paths) and not changed[end + 1]:
            end += 1

        # The first frames have no steady frame before them to hold
        still = start == 0 or end - start + 1 >= STILL_FRAMES
        for index in range(start, end + 1):
            target = steadied / paths[index].name
            if still:
                shutil.copy(paths[end], target)
            else:
                hold(steadied / paths[index - 1].name, paths[index], target)

        start = end + 1


def make_gif(frames, work):
    steadied = work / "steadied"
    steadied.mkdir(exist_ok=True)
    steady(frames, steadied)

    palette = work / "palette.png"
    steady_frames = ["-framerate", str(FPS), "-i", str(steadied / FRAME_NAMES)]
    # The palette favours the colours of what changes, such as the keys lighting
    # up, and there's no dithering, whose dots would differ from frame to frame
    ffmpeg(*steady_frames, "-vf", f"palettegen=max_colors={COLOURS}:stats_mode=diff", str(palette))
    ffmpeg(*steady_frames, "-i", str(palette), "-lavfi",
           "[0:v][1:v]paletteuse=dither=none:diff_mode=rectangle", "-loop", "0", str(OUTPUT))


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

    frames = tmp / "frames"
    frames.mkdir()
    cut(movie, shown_until, results_from, frames)
    make_gif(frames, tmp)
    print(f"Made {OUTPUT} from {movie}")


if __name__ == "__main__":
    main()
