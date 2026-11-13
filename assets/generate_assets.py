#!/usr/bin/env python3
import os
from PIL import Image

def parse_ascii_art(art_str, palette, width, height):
    img = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    lines = [line.strip().split() for line in art_str.strip().split("\n") if line.strip()]
    assert len(lines) == height, f"Expected {height} lines, got {len(lines)}"
    for y, line in enumerate(lines):
        assert len(line) == width, f"Line {y}: Expected {width} cols, got {len(line)}"
        for x, char in enumerate(line):
            color = palette.get(char, (0, 0, 0, 0))
            img.putpixel((x, y), color)
    return img

# =========================================================================
# 16x16 Pixel Art
# =========================================================================

PALETTE_ACTIVE_16 = {
    '.': (0, 0, 0, 0),             # Transparent
    'S': (255, 255, 255, 240),     # Steam bright
    's': (185, 230, 240, 200),     # Steam soft cyan
    'b': (35, 18, 12, 255),        # Dark border
    'C': (240, 98, 65, 255),       # Ceramic warm coral/terracotta
    'H': (255, 160, 135, 255),     # Ceramic highlight
    'D': (180, 50, 25, 255),       # Ceramic deep shadow
    'K': (55, 25, 10, 255),        # Espresso liquid
    'k': (120, 60, 25, 255),       # Crema highlight
    'L': (255, 200, 180, 255),     # Specular glint
}

ART_ACTIVE_16 = """
. . . . . . . . . . . . . . . .
. . . s . . . . . S . . . . . .
. . . . s . . . S . . . . . . .
. . . s . . . . . S . . . . . .
. . b b b b b b b b b . . . . .
. b C C C C C C C C C b b b . .
. b k K K K K K K K C b . . b .
. b C L H C C C C D C b . . b .
. b C H C C C C C D C b b b . .
. b C H C C C C C D C b . . . .
. b C H C C C C C D C b . . . .
. b C H C C C C C D C b . . . .
. . b C C C C C C C b . . . . .
. . . b b b b b b b . . . . . .
. b b b b b b b b b b b . . . .
. . . . . . . . . . . . . . . .
"""

PALETTE_INACTIVE_16 = {
    '.': (0, 0, 0, 0),             # Transparent
    'Z': (120, 195, 245, 240),     # Sleep 'Z' bright
    'z': (70, 160, 220, 200),      # Sleep 'z' soft
    'b': (30, 38, 48, 255),        # Dark slate border
    'C': (85, 110, 125, 255),      # Ceramic cool slate
    'H': (130, 155, 170, 255),     # Ceramic cool highlight
    'D': (50, 70, 82, 255),        # Ceramic shadow
    'K': (25, 30, 35, 255),        # Dark empty interior
    'k': (55, 65, 75, 255),        # Interior rim
    'L': (165, 190, 205, 255),     # Soft reflection
}

ART_INACTIVE_16 = """
. . . . . . . . . . . . . . . .
. . . . . . . . Z Z Z . . . . .
. . . . . . . . . . Z . . . . .
. . . . . z z . . Z . . . . . .
. . b b b b b b b b b . . . . .
. b C C C C C C C C C b b b . .
. b k K K K K K K K C b . . b .
. b C L H C C C C D C b . . b .
. b C H C C C C C D C b b b . .
. b C H C C C C C D C b . . . .
. b C H C C C C C D C b . . . .
. b C H C C C C C D C b . . . .
. . b C C C C C C C b . . . . .
. . . b b b b b b b . . . . . .
. b b b b b b b b b b b . . . .
. . . . . . . . . . . . . . . .
"""

# =========================================================================
# 32x32 Pixel Art (High DPI & Crisp Details)
# =========================================================================

PALETTE_ACTIVE_32 = {
    '.': (0, 0, 0, 0),
    'W': (255, 255, 255, 250),     # Pure white steam/shine
    'S': (220, 245, 255, 220),     # Bright steam
    's': (160, 215, 235, 180),     # Soft steam
    't': (120, 185, 210, 120),     # Fading steam wisp
    'b': (30, 14, 8, 255),         # Outline dark
    'C': (245, 95, 55, 255),       # Ceramic body warm vibrant orange-red
    'H': (255, 155, 125, 255),     # Ceramic light
    'h': (255, 200, 180, 255),     # Ceramic specular highlight
    'D': (190, 50, 20, 255),       # Ceramic dark
    'd': (140, 30, 10, 255),       # Ceramic deep shadow
    'K': (50, 20, 8, 255),         # Dark espresso
    'k': (95, 45, 18, 255),        # Espresso body
    'F': (155, 85, 35, 255),       # Crema foam
    'f': (185, 115, 55, 255),      # Crema highlight
    'P': (220, 75, 40, 255),       # Saucer plate
}

# 32x32 grid
def draw_active_32():
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    # We can draw the 32x32 pixel art procedurally with pixel precision
    # Mug bounds: x in [4..21], handle [21..28], y in [10..26], saucer [2..24] y=27..29
    # Steam trails above y < 10
    pixels = img.load()

    # Steam 1 (left curl rising from x=8..12, y=1..9)
    steam1 = [
        (11, 1, 't'), (12, 1, 's'),
        (12, 2, 'S'), (13, 2, 's'),
        (13, 3, 'W'), (12, 3, 'S'),
        (11, 4, 'S'), (10, 4, 's'),
        (9, 5, 'W'), (10, 5, 'S'),
        (8, 6, 'W'), (9, 6, 'S'),
        (9, 7, 'S'), (10, 7, 's'),
        (10, 8, 'W'), (11, 8, 'S'),
        (11, 9, 'W'), (12, 9, 'S'),
    ]
    for x, y, col in steam1:
        pixels[x, y] = PALETTE_ACTIVE_32[col]

    # Steam 2 (right curl rising from x=15..18, y=2..9)
    steam2 = [
        (17, 2, 't'), (18, 2, 's'),
        (18, 3, 'S'), (19, 3, 's'),
        (19, 4, 'W'), (18, 4, 'S'),
        (17, 5, 'S'), (16, 5, 's'),
        (16, 6, 'W'), (17, 6, 'S'),
        (17, 7, 'S'), (18, 7, 's'),
        (18, 8, 'W'), (19, 8, 'S'),
        (17, 9, 'W'), (18, 9, 'S'),
    ]
    for x, y, col in steam2:
        pixels[x, y] = PALETTE_ACTIVE_32[col]

    # Rim top outline (y=10)
    for x in range(6, 21):
        pixels[x, 10] = PALETTE_ACTIVE_32['b']

    # Rim / Opening (y=11, 12)
    # Liquid ellipse & mug top
    pixels[5, 11] = PALETTE_ACTIVE_32['b']
    for x in range(6, 21):
        pixels[x, 11] = PALETTE_ACTIVE_32['H']
    pixels[21, 11] = PALETTE_ACTIVE_32['b']

    pixels[4, 12] = PALETTE_ACTIVE_32['b']
    pixels[5, 12] = PALETTE_ACTIVE_32['H']
    # Coffee surface (espresso & crema)
    for x in range(6, 10):
        pixels[x, 12] = PALETTE_ACTIVE_32['F']
    pixels[10, 12] = PALETTE_ACTIVE_32['f']
    for x in range(11, 18):
        pixels[x, 12] = PALETTE_ACTIVE_32['K']
    for x in range(18, 21):
        pixels[x, 12] = PALETTE_ACTIVE_32['k']
    pixels[21, 12] = PALETTE_ACTIVE_32['D']
    pixels[22, 12] = PALETTE_ACTIVE_32['b']

    # Coffee second row (y=13)
    pixels[4, 13] = PALETTE_ACTIVE_32['b']
    pixels[5, 13] = PALETTE_ACTIVE_32['C']
    for x in range(6, 9):
        pixels[x, 13] = PALETTE_ACTIVE_32['f']
    for x in range(9, 14):
        pixels[x, 13] = PALETTE_ACTIVE_32['F']
    for x in range(14, 20):
        pixels[x, 13] = PALETTE_ACTIVE_32['K']
    pixels[20, 13] = PALETTE_ACTIVE_32['k']
    pixels[21, 13] = PALETTE_ACTIVE_32['D']
    pixels[22, 13] = PALETTE_ACTIVE_32['b']

    # Mug Body (y=14..24)
    # Handle attaches at y=13..21, extending to x=27
    for y in range(14, 24):
        pixels[4, y] = PALETTE_ACTIVE_32['b']
        pixels[5, y] = PALETTE_ACTIVE_32['C']
        pixels[6, y] = PALETTE_ACTIVE_32['h']
        pixels[7, y] = PALETTE_ACTIVE_32['H']
        pixels[8, y] = PALETTE_ACTIVE_32['H']
        for x in range(9, 17):
            pixels[x, y] = PALETTE_ACTIVE_32['C']
        for x in range(17, 20):
            pixels[x, y] = PALETTE_ACTIVE_32['D']
        pixels[20, y] = PALETTE_ACTIVE_32['d']
        pixels[21, y] = PALETTE_ACTIVE_32['D']
        pixels[22, y] = PALETTE_ACTIVE_32['b']

    # Specular shine vertical strip on cup
    for y in range(15, 22):
        pixels[6, y] = PALETTE_ACTIVE_32['W']

    # Mug Handle:
    # Outer curve: y=13..21, x=22..28
    # Top arc
    for x in range(23, 27):
        pixels[x, 13] = PALETTE_ACTIVE_32['b']
    pixels[27, 14] = PALETTE_ACTIVE_32['b']
    pixels[28, 15] = PALETTE_ACTIVE_32['b']
    pixels[28, 16] = PALETTE_ACTIVE_32['b']
    pixels[28, 17] = PALETTE_ACTIVE_32['b']
    pixels[28, 18] = PALETTE_ACTIVE_32['b']
    pixels[27, 19] = PALETTE_ACTIVE_32['b']
    pixels[26, 20] = PALETTE_ACTIVE_32['b']
    for x in range(23, 26):
        pixels[x, 21] = PALETTE_ACTIVE_32['b']

    # Handle inner fill & highlight
    for x in range(23, 26):
        pixels[x, 14] = PALETTE_ACTIVE_32['H']
    pixels[26, 14] = PALETTE_ACTIVE_32['b']
    pixels[27, 15] = PALETTE_ACTIVE_32['H']
    pixels[27, 16] = PALETTE_ACTIVE_32['C']
    pixels[27, 17] = PALETTE_ACTIVE_32['C']
    pixels[27, 18] = PALETTE_ACTIVE_32['D']
    pixels[26, 19] = PALETTE_ACTIVE_32['D']
    for x in range(23, 26):
        pixels[x, 20] = PALETTE_ACTIVE_32['D']

    # Handle inner hole border
    for x in range(23, 25):
        pixels[x, 15] = PALETTE_ACTIVE_32['b']
    pixels[25, 15] = PALETTE_ACTIVE_32['b']
    pixels[26, 16] = PALETTE_ACTIVE_32['b']
    pixels[26, 17] = PALETTE_ACTIVE_32['b']
    pixels[25, 18] = PALETTE_ACTIVE_32['b']
    for x in range(23, 25):
        pixels[x, 19] = PALETTE_ACTIVE_32['b']

    # Mug Bottom curve (y=24, 25)
    pixels[5, 24] = PALETTE_ACTIVE_32['b']
    for x in range(6, 21):
        pixels[x, 24] = PALETTE_ACTIVE_32['D']
    pixels[21, 24] = PALETTE_ACTIVE_32['b']

    pixels[6, 25] = PALETTE_ACTIVE_32['b']
    for x in range(7, 20):
        pixels[x, 25] = PALETTE_ACTIVE_32['d']
    pixels[20, 25] = PALETTE_ACTIVE_32['b']

    # Saucer plate (y=26..28, x=2..25)
    for x in range(6, 21):
        pixels[x, 26] = PALETTE_ACTIVE_32['b']

    for x in range(3, 24):
        pixels[x, 27] = PALETTE_ACTIVE_32['b']
    pixels[2, 28] = PALETTE_ACTIVE_32['b']
    pixels[3, 28] = PALETTE_ACTIVE_32['h']
    for x in range(4, 18):
        pixels[x, 28] = PALETTE_ACTIVE_32['H']
    for x in range(18, 23):
        pixels[x, 28] = PALETTE_ACTIVE_32['D']
    pixels[23, 28] = PALETTE_ACTIVE_32['d']
    pixels[24, 28] = PALETTE_ACTIVE_32['b']

    for x in range(4, 23):
        pixels[x, 29] = PALETTE_ACTIVE_32['b']

    return img

def draw_inactive_32():
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    pixels = img.load()

    # Sleepy Palette:
    P = {
        'Z1': (140, 215, 255, 250), # Big Z bright
        'Z2': (90, 180, 235, 220),  # Big Z mid
        'z1': (100, 190, 240, 200), # Small z bright
        'z2': (65, 150, 210, 170),  # Small z mid
        'b': (24, 32, 40, 255),     # Cool slate border
        'C': (85, 110, 125, 255),   # Cool slate body
        'H': (135, 160, 175, 255),  # Slate light
        'h': (175, 195, 210, 255),  # Slate highlight
        'W': (210, 230, 245, 240),  # Specular glint
        'D': (55, 75, 88, 255),     # Slate dark
        'd': (38, 52, 62, 255),     # Slate deep shadow
        'K': (22, 28, 34, 255),     # Dark empty interior
        'k': (42, 54, 64, 255),     # Interior shadow
        'P': (75, 98, 112, 255),    # Saucer plate
    }

    # Floating "Z" (Big, at top right x=17..24, y=1..7)
    # Z pattern 6x6
    big_z = [
        (17, 1), (18, 1), (19, 1), (20, 1), (21, 1), (22, 1),
        (21, 2), (20, 3), (19, 4), (18, 5),
        (17, 6), (18, 6), (19, 6), (20, 6), (21, 6), (22, 6)
    ]
    for x, y in big_z:
        pixels[x, y] = P['Z1']
        if (x+1, y) not in big_z and x < 31:
            pass

    # Floating "z" (Small, at mid left x=10..14, y=4..8)
    small_z = [
        (11, 5), (12, 5), (13, 5),
        (12, 6),
        (11, 7), (12, 7), (13, 7)
    ]
    for x, y in small_z:
        pixels[x, y] = P['z1']

    # Rim top outline (y=10)
    for x in range(6, 21):
        pixels[x, 10] = P['b']

    # Rim / Opening (y=11, 12)
    pixels[5, 11] = P['b']
    for x in range(6, 21):
        pixels[x, 11] = P['H']
    pixels[21, 11] = P['b']

    pixels[4, 12] = P['b']
    pixels[5, 12] = P['H']
    # Interior (dark/empty)
    for x in range(6, 21):
        pixels[x, 12] = P['K']
    pixels[21, 12] = P['D']
    pixels[22, 12] = P['b']

    # Coffee second row (y=13)
    pixels[4, 13] = P['b']
    pixels[5, 13] = P['C']
    for x in range(6, 21):
        pixels[x, 13] = P['k']
    pixels[21, 13] = P['D']
    pixels[22, 13] = P['b']

    # Mug Body (y=14..24)
    for y in range(14, 24):
        pixels[4, y] = P['b']
        pixels[5, y] = P['C']
        pixels[6, y] = P['h']
        pixels[7, y] = P['H']
        pixels[8, y] = P['H']
        for x in range(9, 17):
            pixels[x, y] = P['C']
        for x in range(17, 20):
            pixels[x, y] = P['D']
        pixels[20, y] = P['d']
        pixels[21, y] = P['D']
        pixels[22, y] = P['b']

    # Specular shine
    for y in range(15, 22):
        pixels[6, y] = P['W']

    # Mug Handle:
    for x in range(23, 27):
        pixels[x, 13] = P['b']
    pixels[27, 14] = P['b']
    pixels[28, 15] = P['b']
    pixels[28, 16] = P['b']
    pixels[28, 17] = P['b']
    pixels[28, 18] = P['b']
    pixels[27, 19] = P['b']
    pixels[26, 20] = P['b']
    for x in range(23, 26):
        pixels[x, 21] = P['b']

    for x in range(23, 26):
        pixels[x, 14] = P['H']
    pixels[26, 14] = P['b']
    pixels[27, 15] = P['H']
    pixels[27, 16] = P['C']
    pixels[27, 17] = P['C']
    pixels[27, 18] = P['D']
    pixels[26, 19] = P['D']
    for x in range(23, 26):
        pixels[x, 20] = P['D']

    for x in range(23, 25):
        pixels[x, 15] = P['b']
    pixels[25, 15] = P['b']
    pixels[26, 16] = P['b']
    pixels[26, 17] = P['b']
    pixels[25, 18] = P['b']
    for x in range(23, 25):
        pixels[x, 19] = P['b']

    # Mug Bottom curve (y=24, 25)
    pixels[5, 24] = P['b']
    for x in range(6, 21):
        pixels[x, 24] = P['D']
    pixels[21, 24] = P['b']

    pixels[6, 25] = P['b']
    for x in range(7, 20):
        pixels[x, 25] = P['d']
    pixels[20, 25] = P['b']

    # Saucer plate (y=26..28)
    for x in range(6, 21):
        pixels[x, 26] = P['b']

    for x in range(3, 24):
        pixels[x, 27] = P['b']
    pixels[2, 28] = P['b']
    pixels[3, 28] = P['h']
    for x in range(4, 18):
        pixels[x, 28] = P['H']
    for x in range(18, 23):
        pixels[x, 28] = P['D']
    pixels[23, 28] = P['d']
    pixels[24, 28] = P['b']

    for x in range(4, 23):
        pixels[x, 29] = P['b']

    return img

def main():
    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    assets_dir = os.path.join(base_dir, "assets")
    src_dir = os.path.join(base_dir, "src")
    os.makedirs(assets_dir, exist_ok=True)
    os.makedirs(src_dir, exist_ok=True)

    img_active_16 = parse_ascii_art(ART_ACTIVE_16, PALETTE_ACTIVE_16, 16, 16)
    img_inactive_16 = parse_ascii_art(ART_INACTIVE_16, PALETTE_INACTIVE_16, 16, 16)

    img_active_32 = draw_active_32()
    img_inactive_32 = draw_inactive_32()

    # 48x48 versions for Dialog & Explorer icon (clean nearest-neighbor scale for authentic pixel art look)
    img_active_48 = img_active_16.resize((48, 48), Image.Resampling.NEAREST)
    img_inactive_48 = img_inactive_16.resize((48, 48), Image.Resampling.NEAREST)

    # Save PNGs
    img_active_16.save(os.path.join(assets_dir, "active_16.png"))
    img_inactive_16.save(os.path.join(assets_dir, "inactive_16.png"))
    img_active_32.save(os.path.join(assets_dir, "active_32.png"))
    img_inactive_32.save(os.path.join(assets_dir, "inactive_32.png"))
    img_active_48.save(os.path.join(assets_dir, "active_48.png"))
    img_inactive_48.save(os.path.join(assets_dir, "inactive_48.png"))

    # Save Windows .ICO files (with multi-resolution 16, 32, 48)
    img_active_32.save(os.path.join(assets_dir, "active.ico"), format="ICO", sizes=[(16, 16), (32, 32), (48, 48)])
    img_inactive_32.save(os.path.join(assets_dir, "inactive.ico"), format="ICO", sizes=[(16, 16), (32, 32), (48, 48)])

    print("Generated PNG and ICO assets successfully.")

    # Generate Rust source code with raw RGBA byte arrays
    def to_rust_bytes(img):
        raw = img.tobytes("raw", "RGBA")
        lines = []
        for i in range(0, len(raw), 16):
            chunk = raw[i:i+16]
            lines.append("    " + ", ".join(f"0x{b:02X}" for b in chunk) + ",")
        return "\n".join(lines)

    rust_code = f"""// Auto-generated handcrafted pixel art assets for Caffeine
// Generated by assets/generate_assets.py

pub const ACTIVE_16_RGBA: [u8; 16 * 16 * 4] = [
{to_rust_bytes(img_active_16)}
];

pub const INACTIVE_16_RGBA: [u8; 16 * 16 * 4] = [
{to_rust_bytes(img_inactive_16)}
];

pub const ACTIVE_32_RGBA: [u8; 32 * 32 * 4] = [
{to_rust_bytes(img_active_32)}
];

pub const INACTIVE_32_RGBA: [u8; 32 * 32 * 4] = [
{to_rust_bytes(img_inactive_32)}
];

pub const ACTIVE_48_RGBA: [u8; 48 * 48 * 4] = [
{to_rust_bytes(img_active_48)}
];

pub const INACTIVE_48_RGBA: [u8; 48 * 48 * 4] = [
{to_rust_bytes(img_inactive_48)}
];
"""
    with open(os.path.join(src_dir, "pixel_data.rs"), "w") as f:
        f.write(rust_code)
    print("Generated src/pixel_data.rs successfully.")

if __name__ == "__main__":
    main()
