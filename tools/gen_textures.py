#!/usr/bin/env python3
"""Draw this mod's textures, 16x16 each, all original.

Tom's Simple Storage's own art is not used: these are plain copper-and-stone
tiles drawn here with PIL, so the mod may be redistributed under MIT.

Run: python tools/gen_textures.py
"""
import os
import random

from PIL import Image, ImageDraw

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(os.path.dirname(HERE), "content", "assets", "toms_storage", "textures", "block")


def noise(img, base, spread, seed):
    rnd = random.Random(seed)
    px = img.load()
    for y in range(16):
        for x in range(16):
            d = rnd.randint(-spread, spread)
            px[x, y] = tuple(max(0, min(255, c + d)) for c in base) + (255,)


def frame(d, rect, fill, outline):
    d.rectangle(rect, fill=fill, outline=outline)


def terminal_side():
    img = Image.new("RGBA", (16, 16))
    noise(img, (74, 78, 84), 10, 11)
    d = ImageDraw.Draw(img)
    # Copper rim around a dark screen with three green rows of "items".
    frame(d, (1, 1, 14, 14), None, (176, 116, 63))
    frame(d, (2, 3, 13, 11), (24, 30, 34), (120, 78, 44))
    rnd = random.Random(7)
    for row in range(3):
        y = 4 + row * 3
        for col in range(4):
            x = 3 + col * 3
            if rnd.random() < 0.8:
                d.rectangle([x, y, x + 1, y + 1], fill=(96, 200, 140))
    d.rectangle([4, 12, 11, 13], fill=(52, 56, 62), outline=(120, 78, 44))
    return img


def terminal_top():
    img = Image.new("RGBA", (16, 16))
    noise(img, (88, 92, 98), 8, 12)
    d = ImageDraw.Draw(img)
    frame(d, (0, 0, 15, 15), None, (60, 64, 70))
    d.rectangle([2, 2, 13, 13], fill=(104, 108, 114), outline=(176, 116, 63))
    d.rectangle([6, 6, 9, 9], fill=(70, 74, 80), outline=(120, 78, 44))
    return img


def terminal_bottom():
    img = Image.new("RGBA", (16, 16))
    noise(img, (66, 70, 76), 8, 13)
    d = ImageDraw.Draw(img)
    frame(d, (0, 0, 15, 15), None, (48, 52, 58))
    for i in range(0, 16, 4):
        d.line([(i, 0), (i, 15)], fill=(58, 62, 68))
        d.line([(0, i), (15, i)], fill=(58, 62, 68))
    return img


def crafting_side():
    img = Image.new("RGBA", (16, 16))
    noise(img, (74, 78, 84), 10, 31)
    d = ImageDraw.Draw(img)
    # The terminal's copper rim, with a small 3x3 grid where the screen is.
    frame(d, (1, 1, 14, 14), None, (176, 116, 63))
    frame(d, (2, 3, 13, 12), (44, 34, 26), (120, 78, 44))
    for i in range(3):
        for j in range(3):
            x = 3 + i * 4
            y = 4 + j * 3
            d.rectangle([x, y, x + 2, y + 1], fill=(160, 112, 66), outline=(96, 62, 34))
    return img


def crafting_top():
    img = Image.new("RGBA", (16, 16))
    noise(img, (88, 92, 98), 8, 32)
    d = ImageDraw.Draw(img)
    frame(d, (0, 0, 15, 15), None, (60, 64, 70))
    # A flat 3x3 crafting grid cut into the top.
    d.rectangle([1, 1, 14, 14], fill=(104, 108, 114), outline=(176, 116, 63))
    d.rectangle([2, 2, 13, 13], fill=(70, 52, 38), outline=(120, 78, 44))
    for i in (1, 2):
        x = 2 + i * 4
        y = 2 + i * 4
        d.line([(x, 2), (x, 13)], fill=(120, 78, 44))
        d.line([(2, y), (13, y)], fill=(120, 78, 44))
    return img


def crate_side():
    img = Image.new("RGBA", (16, 16))
    noise(img, (150, 110, 64), 10, 41)
    d = ImageDraw.Draw(img)
    # Vertical planks with a copper band top and bottom, and a gap between two.
    for x in (0, 5, 10, 15):
        d.line([(x, 0), (x, 15)], fill=(104, 74, 42))
    d.line([(4, 0), (4, 15)], fill=(186, 140, 86))
    d.line([(9, 0), (9, 15)], fill=(186, 140, 86))
    d.rectangle([0, 0, 15, 2], fill=(176, 116, 63), outline=(120, 78, 44))
    d.rectangle([0, 13, 15, 15], fill=(176, 116, 63), outline=(120, 78, 44))
    d.point((2, 7), fill=(214, 160, 96))
    d.point((12, 7), fill=(214, 160, 96))
    return img


def crate_top():
    img = Image.new("RGBA", (16, 16))
    noise(img, (150, 110, 64), 10, 42)
    d = ImageDraw.Draw(img)
    frame(d, (0, 0, 15, 15), None, (104, 74, 42))
    # Open: an empty dark inside, the rim of the crate around it.
    d.rectangle([2, 2, 13, 13], fill=(30, 24, 20), outline=(120, 78, 44))
    d.line([(2, 2), (13, 2)], fill=(186, 140, 86))
    d.line([(2, 3), (13, 3)], fill=(120, 86, 50))
    d.rectangle([0, 0, 1, 15], fill=(176, 116, 63))
    d.rectangle([14, 0, 15, 15], fill=(176, 116, 63))
    return img


def crate_bottom():
    img = Image.new("RGBA", (16, 16))
    noise(img, (140, 102, 58), 10, 43)
    d = ImageDraw.Draw(img)
    frame(d, (0, 0, 15, 15), None, (96, 68, 38))
    for i in range(0, 16, 5):
        d.line([(i, 0), (i, 15)], fill=(104, 74, 42))
    return img


def connector():
    img = Image.new("RGBA", (16, 16))
    noise(img, (150, 100, 56), 12, 21)
    d = ImageDraw.Draw(img)
    d.rectangle([0, 0, 15, 15], outline=(96, 62, 34))
    # Rivets on the corners of the 8px body.
    for x, y in ((3, 3), (12, 3), (3, 12), (12, 12)):
        d.point((x, y), fill=(214, 160, 96))
    d.rectangle([5, 5, 10, 10], fill=(176, 116, 63), outline=(96, 62, 34))
    return img


def cable():
    img = Image.new("RGBA", (16, 16))
    noise(img, (176, 116, 63), 14, 22)
    d = ImageDraw.Draw(img)
    for y in (0, 15):
        d.line([(0, y), (15, y)], fill=(120, 78, 44))
    d.line([(0, 7), (15, 7)], fill=(206, 148, 88))
    d.line([(0, 8), (15, 8)], fill=(140, 92, 52))
    return img


def main():
    os.makedirs(OUT, exist_ok=True)
    for name, img in (
        ("storage_terminal_side", terminal_side()),
        ("storage_terminal_top", terminal_top()),
        ("storage_terminal_bottom", terminal_bottom()),
        ("crafting_terminal_side", crafting_side()),
        ("crafting_terminal_top", crafting_top()),
        ("crafting_terminal_bottom", terminal_bottom()),
        ("open_crate_side", crate_side()),
        ("open_crate_top", crate_top()),
        ("open_crate_bottom", crate_bottom()),
        ("inventory_connector", connector()),
        ("inventory_cable", cable()),
    ):
        path = os.path.join(OUT, name + ".png")
        img.save(path)
        print(os.path.relpath(path, os.path.dirname(HERE)), img.size)


if __name__ == "__main__":
    main()
