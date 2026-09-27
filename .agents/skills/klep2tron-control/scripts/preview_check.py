#!/usr/bin/env python3
"""Report which editor preview thumbnails are visible in a screenshot.

Usage:
    python3 preview_check.py shot.png [shot2.png ...]

Prints one line per image, e.g. `[5-555]` meaning:
    position 1 = Cube, 2 = Wedge S, 3 = Wedge W, 4 = Wedge N, 5 = Wedge E
    `5` = thumbnail rendered (green pixels found), `-` = blank.

Needs Pillow. On this machine:
    python3 -m venv /tmp/imgvenv && /tmp/imgvenv/bin/pip install pillow
    /tmp/imgvenv/bin/python preview_check.py shot.png
"""
import sys
from PIL import Image

# x windows of the five thumbnails in the top panel (2560x1440 window).
WINDOWS = [(810, 920), (978, 1080), (1144, 1250), (1315, 1420), (1480, 1590)]


def present(px, x0, x1):
    for x in range(x0, x1):
        for y in range(0, 200, 2):
            r, g, b = px[x, y]
            if g > 60 and g > r + 20 and g > b + 20:
                return True
    return False


def main(paths):
    for p in paths:
        im = Image.open(p).convert("RGB")
        px = im.load()
        bits = "".join("5" if present(px, a, b) else "-" for a, b in WINDOWS)
        print(f"{p.split('/')[-1]:16} [{bits}]")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)
    main(sys.argv[1:])
