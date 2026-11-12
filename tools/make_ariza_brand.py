"""Draw Ariza Studio's logo: the Oyenza Studio mark, with the name changed everywhere.

Same visual system as Oyenza Studio (docs/brand): the orange tile, the island hanging from its top edge
playing music, the white cat below it. Only the wordmark is new: "Ariza Studio", set the way the site sets
"Oyenza Studio" -- semibold, -0.01em tracking, ink #171717 -- and turned into outlines so the files need no
font installed.

Run: python3 tools/make_ariza_brand.py  (needs Pillow and fontTools)

Writes brand/ariza/:
  ariza-mark.svg, ariza-mark-*.png     the tile alone
  ariza-studio-icon.ico                app, window and tray icon, every size drawn on its own
  ariza-studio-lockup{,-dark}.svg/png  tile + wordmark, for light and for dark backgrounds
  preview.png                          the sheet: both lockups, the mark, the palette

The tile is drawn by tools/make_brand.py; this file only changes its <title> and its ids, then scales it, so
the two never drift apart.

The wordmark is Inter (SIL OFL), the first fallback in Oyenza Studio's own font stack. SF Pro Text, which the
site uses, may not be redistributed as outlines.
"""
import math
import re
import sys
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFont
from fontTools.pens.recordingPen import DecomposingRecordingPen
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "brand" / "ariza"
CACHE = ROOT / ".cache"
sys.path.insert(0, str(ROOT / "tools"))

import make_brand as mb  # noqa: E402  the tile: geometry, gradient, the cat

ICON = 1024.0  # the lockup's unit: the tile's box is ICON wide, so the tile svg maps 1:1
INSET = mb.INSET * ICON  # the tile's visible edge, inside its box
CAP = 0.42  # cap height, as a fraction of the icon
GAP = 0.60  # space between the tile's ink and the wordmark's ink, in cap heights
TRACK = -0.01  # letter-spacing, in em -- the site's .side-brand
INK, INK_DARK = "#171717", "#FFFFFF"
NAME = "Ariza Studio"

FONT_CANDIDATES = [
    Path.home() / "Library/Fonts/Inter-VariableFont_opsz,wght.ttf",
    Path("/Library/Fonts/Inter-VariableFont_opsz,wght.ttf"),
    OUT / "Inter-VariableFont_opsz,wght.ttf",
]


# --- the wordmark ---------------------------------------------------------------------------------

def _static_font(wght: int, opsz: int = 32) -> TTFont:
    """Inter at one weight, cached as a static font: the variable one cannot be drawn per-glyph."""
    path = CACHE / f"Inter-{wght}.ttf"
    if path.exists():
        return TTFont(path)
    src = next((p for p in FONT_CANDIDATES if p.exists()), None)
    if src is None:
        sys.exit("Inter not found; looked for:\n  " + "\n  ".join(map(str, FONT_CANDIDATES)))
    CACHE.mkdir(exist_ok=True)
    font = instantiateVariableFont(TTFont(src), {"wght": wght, "opsz": opsz}, updateFontNames=False)
    font.save(path)
    return font


def _contours(font: TTFont, glyph: str) -> list[list[tuple]]:
    """A glyph's outlines, y up: ("L", a, b) | ("Q", a, c, b) | ("C", a, c1, c2, b)."""
    glyphs = font.getGlyphSet()
    pen = DecomposingRecordingPen(glyphs)  # letters such as "i" are composites
    glyphs[glyph].draw(pen)
    out, cur, last, start = [], [], None, None
    for op, args in pen.value:
        if op == "moveTo":
            if cur:
                out.append(cur)
            cur, start, last = [], args[0], args[0]
        elif op == "lineTo":
            cur.append(("L", last, args[0]))
            last = args[0]
        elif op == "qCurveTo":
            pts = list(args)
            if pts[-1] is None:  # TrueType's sparse contour: it closes back onto the start
                pts[-1] = start
            offs, end, prev = pts[:-1], pts[-1], last
            if len(offs) == 1:
                cur.append(("Q", prev, offs[0], end))
            else:  # several off-curve points: the on-curve ones sit at their midpoints
                for a, b in zip(offs, offs[1:]):
                    mid = ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2)
                    cur.append(("Q", prev, a, mid))
                    prev = mid
                cur.append(("Q", prev, offs[-1], end))
            last = end
        elif op == "curveTo":
            pts, prev = list(args), last
            for i in range(0, len(pts) - 1, 2):
                cur.append(("C", prev, pts[i], pts[i + 1], pts[-1]))
            last = pts[-1]
    if cur:
        out.append(cur)
    return out


def _bounds(contours) -> tuple[float, float, float, float]:
    """The ink box of a glyph's contours (y up, font units)."""
    pts = [pt for contour in contours for seg in contour for pt in seg[1:]]
    return (min(p[0] for p in pts), min(p[1] for p in pts),
            max(p[0] for p in pts), max(p[1] for p in pts))


def _flatten(contour, steps: int = 18) -> list[tuple[float, float]]:
    """One contour as a polyline, sampling its quadratics and cubics."""
    pts = [contour[0][1]]
    for s in contour:
        if s[0] == "L":
            pts.append(s[2])
        elif s[0] == "Q":
            (x0, y0), (cx, cy), (x1, y1) = s[1], s[2], s[3]
            for i in range(1, steps + 1):
                u = i / steps
                pts.append(((1 - u) ** 2 * x0 + 2 * (1 - u) * u * cx + u * u * x1,
                            (1 - u) ** 2 * y0 + 2 * (1 - u) * u * cy + u * u * y1))
        else:
            (x0, y0), (a, b), (c, d), (x1, y1) = s[1], s[2], s[3], s[4]
            for i in range(1, steps + 1):
                u = i / steps
                pts.append(((1 - u) ** 3 * x0 + 3 * (1 - u) ** 2 * u * a + 3 * (1 - u) * u * u * c + u ** 3 * x1,
                            (1 - u) ** 3 * y0 + 3 * (1 - u) ** 2 * u * b + 3 * (1 - u) * u * u * d + u ** 3 * y1))
    return pts


class Wordmark:
    """NAME laid out with Inter's advances and TRACK tracking, in font units, baseline at y = 0."""

    def __init__(self, wght: int = 600):
        self.font = _static_font(wght)
        self.upm = self.font["head"].unitsPerEm
        self.cap = self.font["OS/2"].sCapHeight
        cmap, hmtx = self.font.getBestCmap(), self.font["hmtx"]
        self.shapes = []  # (letter, contours, x offset, advance)
        x = 0.0
        for ch in NAME:
            glyph = cmap[ord(ch)]
            contours = _contours(self.font, glyph)
            if ch != " " and not contours:  # composites are decomposed; anything else is a bug
                raise ValueError(f"{ch!r} ({glyph}) drew no contours")
            self.shapes.append((ch, contours, x, hmtx[glyph][0]))
            x += hmtx[glyph][0] + TRACK * self.upm
        self.ink = (min(_bounds(c)[0] + dx for _, c, dx, _ in self.shapes if c),
                    min(_bounds(c)[1] for _, c, _, _ in self.shapes if c),
                    max(_bounds(c)[2] + dx for _, c, dx, _ in self.shapes if c),
                    max(_bounds(c)[3] for _, c, _, _ in self.shapes if c))

    def size_for_cap(self, cap: float) -> float:
        return cap * self.upm / self.cap

    def placed(self, size: float, x0: float, baseline: float) -> list[list[tuple[float, float]]]:
        """Every contour, flattened to polygons in drawing space (y down)."""
        k = size / self.upm
        return [[(x0 + (p[0] + dx) * k, baseline - p[1] * k) for p in _flatten(contour)]
                for _, contours, dx, _ in self.shapes for contour in contours]

    def path(self, size: float, x0: float, baseline: float) -> str:
        """Every contour as one svg path in drawing space; TrueType windings punch the counters."""
        k = size / self.upm
        X = lambda v, dx: f"{x0 + (v + dx) * k:.2f}"
        Y = lambda v: f"{baseline - v * k:.2f}"
        out = []
        for _, contours, dx, _ in self.shapes:
            for contour in contours:
                out.append(f"M{X(contour[0][1][0], dx)} {Y(contour[0][1][1])}")
                for s in contour:
                    if s[0] == "L":
                        out.append(f"L{X(s[2][0], dx)} {Y(s[2][1])}")
                    elif s[0] == "Q":
                        out.append(f"Q{X(s[2][0], dx)} {Y(s[2][1])} {X(s[3][0], dx)} {Y(s[3][1])}")
                    else:
                        out.append(f"C{X(s[2][0], dx)} {Y(s[2][1])} {X(s[3][0], dx)} {Y(s[3][1])} "
                                   f"{X(s[4][0], dx)} {Y(s[4][1])}")
                out.append("Z")
        return "".join(out)

    def pair_gaps(self) -> list[tuple[str, float]]:
        """Ink gap between neighbouring letters, in em: the check that tracking did not collide them."""
        gaps, prev = [], None
        for ch, contours, dx, _ in self.shapes:
            if not contours:
                prev = None
                continue
            b = _bounds(contours)
            if prev is not None:
                gaps.append((f"{prev[0]}{ch}", (b[0] + dx - prev[1]) / self.upm))
            prev = (ch, b[2] + dx)
        return gaps


def fill_nonzero(mask: Image.Image, polys: list[list[tuple[float, float]]], value: int = 255) -> None:
    """Scanline fill with the nonzero winding rule: how TrueType fills its outlines.

    The letters are outlined the way a font is, so a stroke that crosses another one (the bar of "A",
    the stem of "t") has two contours over the same pixels. XOR would punch a hole there; counting the
    windings fills the union, and keeps the counters of "a" and "o" empty.
    """
    W, H = mask.size
    rows: dict[int, list[tuple[float, int]]] = {}
    for poly in polys:
        for (x0, y0), (x1, y1) in zip(poly, poly[1:] + poly[:1]):
            if y0 == y1:
                continue
            direction = 1 if y1 > y0 else -1
            lo, hi = min(y0, y1), max(y0, y1)
            for y in range(max(0, math.floor(lo)), min(H, math.ceil(hi))):
                yc = y + 0.5
                if lo <= yc < hi:
                    rows.setdefault(y, []).append((x0 + (yc - y0) * (x1 - x0) / (y1 - y0), direction))
    d = ImageDraw.Draw(mask)
    for y, cuts in rows.items():
        cuts.sort()
        winding, start = 0, None
        for x, direction in cuts:
            before = winding
            winding += direction
            if before == 0 and winding != 0:
                start = x
            elif before != 0 and winding == 0 and start is not None:
                d.rectangle([math.floor(start), y, math.ceil(x) - 1, y], fill=value)
                start = None


# --- the tile ------------------------------------------------------------------------------------

def icon_inner() -> str:
    """The tile's svg body from make_brand, with this brand's title and ids."""
    svg = mb.logo_svg().replace("<title>Oyenza Studio</title>", "")
    body = re.sub(r"^<svg[^>]*>", "", svg).rsplit("</svg>", 1)[0]
    body = body.replace('id="tile"', 'id="ariza-tile"').replace("url(#tile)", "url(#ariza-tile)")
    return body.replace('id="cover"', 'id="ariza-cover"').replace("url(#cover)", "url(#ariza-cover)")


def icon_svg() -> str:
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {ICON:g} {ICON:g}" '
            f'width="{ICON:g}" height="{ICON:g}"><title>Ariza Studio</title>{icon_inner()}</svg>\n')


# --- layout --------------------------------------------------------------------------------------

def _union(a, b):
    return min(a[0], b[0]), min(a[1], b[1]), max(a[2], b[2]), max(a[3], b[3])


class Plan:
    """Where the tile and the wordmark sit, and the tight box around their ink.

    Units are lockup units: the tile's box is ICON square at (0, 0).
    """

    def __init__(self, mark: Wordmark):
        self.size = mark.size_for_cap(CAP * ICON)
        k = self.size / mark.upm
        gap = GAP * CAP * ICON - INSET - mark.ink[0] * k  # ink to ink, less both side bearings
        self.x0, self.baseline = ICON + gap, ICON / 2 + (mark.ink[1] + mark.ink[3]) / 2 * k
        text = (self.x0 + mark.ink[0] * k, self.baseline - mark.ink[3] * k,
                self.x0 + mark.ink[2] * k, self.baseline - mark.ink[1] * k)
        self.box = _union((INSET, INSET, ICON - INSET, ICON - INSET), text)
        self.text_w = text[2] - text[0]
        self.width = self.box[2] - self.box[0]
        self.height = self.box[3] - self.box[1]


def lockup_svg(mark: Wordmark, ink: str) -> str:
    p = Plan(mark)
    x0, y0, x1, y1 = p.box
    tile = icon_inner()  # authored in the tile's own 1024 units, which is what ICON is
    text = f'<path d="{mark.path(p.size, p.x0, p.baseline)}" fill="{ink}"/>'
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{x0:.1f} {y0:.1f} {p.width:.1f} {p.height:.1f}" '
            f'width="{p.width:.1f}" height="{p.height:.1f}"><title>Ariza Studio</title>\n'
            f'{tile}\n{text}\n</svg>\n')


def lockup_png(mark: Wordmark, ink: str, width: int = 2048, ss: int = 3) -> Image.Image:
    """The lockup from the same geometry as the svg: drawn at ss x, then LANCZOS down."""
    p = Plan(mark)
    k = width / p.width * ss
    cw, ch = round(p.width * k), round(p.height * k)
    ox, oy = p.box[0], p.box[1]
    canvas = Image.new("RGBA", (cw, ch), (0, 0, 0, 0))

    tile_size = round(ICON * k)
    tile = mb.draw_logo(tile_size, ss=2)
    canvas.alpha_composite(tile, (-round(ox * k), -round(oy * k)))

    mask = Image.new("L", (cw, ch), 0)
    fill_nonzero(mask, mark.placed(p.size * k, (p.x0 - ox) * k, (p.baseline - oy) * k))
    rgb = Image.new("RGBA", (cw, ch), ink)
    rgb.putalpha(mask)
    canvas.alpha_composite(rgb)
    return canvas.resize((round(width), round(width * p.height / p.width)), Image.LANCZOS)


# --- preview sheet -------------------------------------------------------------------------------

def tracked(draw, xy, text, font, fill, tracking=0.0):
    """Letter-spaced text: PIL has no tracking."""
    x, y = xy
    for ch in text:
        draw.text((x, y), ch, font=font, fill=fill)
        x += draw.textlength(ch, font=font) + tracking


def preview(lab, small, light: Image.Image, dark: Image.Image, icons: dict) -> Image.Image:
    """The sheet: each lockup under its own label, then the mark and the palette."""
    W, H, M = 2400, 1520, 120
    sheet = Image.new("RGBA", (W, H), "#FFFFFF")
    d = ImageDraw.Draw(sheet)

    tracked(d, (M, 90), "ARIZA STUDIO", lab, "#171717", 7)
    tracked(d, (M, 140), "BRAND MARK  ·  THE OYENZA SYSTEM, RENAMED", small, "#86868B", 3)
    d.line([(M, 200), (W - M, 200)], fill="#E8E8ED", width=2)

    # the lockup on light
    tracked(d, (M, 246), "PRIMARY  ·  LIGHT  ·  SMALLEST WIDTH 96 PX", small, "#86868B", 3)
    lw = 1120
    lh = round(light.height * lw / light.width)
    sheet.alpha_composite(light.resize((lw, lh), Image.LANCZOS), ((W - lw) // 2, 290))

    # the lockup on dark
    tracked(d, (M, 592), "ON DARK", small, "#86868B", 3)
    d.rounded_rectangle([M, 632, W - M, 962], radius=36, fill="#0A0A0A")
    dw = 880
    dh = round(dark.height * dw / dark.width)
    sheet.alpha_composite(dark.resize((dw, dh), Image.LANCZOS), ((W - dw) // 2, 632 + (330 - dh) // 2))

    # the mark, at the sizes an icon is asked for
    tracked(d, (M, 1016), "MARK", lab, "#171717", 6)
    x, foot = M + 6, 1262  # every icon sits on the same line, so the captions line up too
    for n in (200, 128, 64, 32):
        sheet.alpha_composite(icons[n], (x, foot - n))
        d.text((x, foot + 18), f"{n} px", font=small, fill="#86868B")
        x += n + 96

    # the palette
    px = 1300
    tracked(d, (px, 1016), "PALETTE", lab, "#171717", 6)
    swatches = [("#F0892E", "Orange, the tile"), ("#0A0A0A", "Island"), ("#FFA24A", "Equalizer"),
                ("#FFB37A", "Sunset, top"), ("#2B4F86", "Sunset, sea"), ("#171717", "Ink")]
    for i, (hexv, name) in enumerate(swatches):
        cx, cy = px + (i % 3) * 300, 1062 + (i // 3) * 190
        d.rounded_rectangle([cx, cy, cx + 72, cy + 72], radius=18, fill=hexv)
        d.text((cx, cy + 84), name, font=small, fill="#171717")
        d.text((cx, cy + 114), hexv.upper(), font=small, fill="#86868B")
        for text in (name, hexv.upper()):  # the column is 300 wide; a longer name would run into the next one
            if d.textlength(text, font=small) > 292:
                print(f"  ! palette label too wide: {text!r}")

    d.line([(M, H - 100), (W - M, H - 100)], fill="#E8E8ED", width=2)
    note = ("tools/make_ariza_brand.py draws every file in this folder.  Wordmark: Inter SemiBold, -1% tracking, "
            "outlined.  Tile: tools/make_brand.py.")
    while d.textlength(note, font=small) > W - 2 * M and len(note) > 40:  # keep the footer inside the margins
        note = note[:note.rindex(" ")] + "."
    d.text((M, H - 74), note, font=small, fill="#AEAEB2")
    return sheet.convert("RGB")


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    mark = Wordmark(600)
    p = Plan(mark)
    print(f"icon {ICON:.0f} · cap {CAP * ICON:.0f} · font size {p.size:.1f} ({p.size / ICON:.3f} x icon)")
    print(f"lockup {p.width:.0f} x {p.height:.0f} units ({p.width / p.height:.2f}:1), "
          f"ink gap {GAP * CAP * ICON:.0f}")
    print("pair gaps, em:", " ".join(f"{n} {g:+.3f}" for n, g in mark.pair_gaps()))

    OUT.joinpath("ariza-mark.svg").write_text(icon_svg())
    pngs = {}
    for n in (1024, 512, 256, 128, 64, 32, 16):
        pngs[n] = mb.draw_logo(n, ss=4 if n >= 512 else 8)
        pngs[n].save(OUT / f"ariza-mark-{n}.png", optimize=True)

    ico = {n: mb.draw_logo(n, ss=8) for n in (16, 20, 24, 32, 40, 48, 64, 96, 128, 256)}
    ico[256].save(OUT / "ariza-studio-icon.ico", format="ICO", sizes=[(n, n) for n in ico],
                  append_images=[ico[n] for n in sorted(ico)[:-1]])

    OUT.joinpath("ariza-studio-lockup.svg").write_text(lockup_svg(mark, INK))
    OUT.joinpath("ariza-studio-lockup-dark.svg").write_text(lockup_svg(mark, INK_DARK))
    light = lockup_png(mark, INK, 2048)
    dark = lockup_png(mark, INK_DARK, 2048)
    light.save(OUT / "ariza-studio-lockup.png", optimize=True)
    dark.save(OUT / "ariza-studio-lockup-dark.png", optimize=True)
    lockup_png(mark, INK, 1024).save(OUT / "ariza-studio-lockup-1024.png", optimize=True)

    lab = ImageFont.truetype(str(CACHE / "Inter-600.ttf"), 34)
    small = ImageFont.truetype(str(CACHE / "Inter-600.ttf"), 26)
    icons = {200: mb.draw_logo(200, ss=6), **{n: pngs[n] for n in (128, 64, 32)}}
    sheet = preview(lab, small, light, dark, icons)
    sheet.save(OUT / "preview.png", optimize=True)

    for f in sorted(OUT.iterdir()):
        print(f"{f.relative_to(ROOT)}  {f.stat().st_size:,} bytes")
    return 0


if __name__ == "__main__":
    sys.exit(main())
