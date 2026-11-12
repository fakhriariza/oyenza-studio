# Ariza Studio brand files

Ariza Studio's mark in the Oyenza Studio system: the same orange tile, the island hanging from its top
edge playing music (album cover and equalizer, as on the real island), the white Oyen below it. Only the
name changed — it reads **Ariza Studio**, set the way the site sets *Oyenza Studio*: semibold, −0.01em
tracking, ink `#171717`.

The tile is drawn by [`tools/make_brand.py`](../../tools/make_brand.py), so it is the same mark, pixel for
pixel; [`tools/make_ariza_brand.py`](../../tools/make_ariza_brand.py) only swaps the `<title>`, the ids and
the wordmark.

| File | What |
|---|---|
| `ariza-mark.svg`, `ariza-mark-{16,32,64,128,256,512,1024}.png` | The tile alone: favicon, app icon, avatar, tray |
| `ariza-studio-icon.ico` | The same tile as a Windows icon, every size drawn on its own (16–256) |
| `ariza-studio-lockup.svg`, `.png`, `-1024.png` | Tile and wordmark, for light backgrounds |
| `ariza-studio-lockup-dark.svg`, `.png` | The same for dark backgrounds; the tile is unchanged, the name turns white |
| `preview.png` | The sheet: both lockups, the mark at 200/128/64/32 px, the palette |

The PNGs are drawn from the same geometry as the SVGs, three times over and then scaled down, so the two
never disagree. They are 2048 and 1024 pixels wide.

## The lockup

Measured in the tile's own units (its box is 1024 square, its visible ink 927.7):

| | |
|---|---|
| Lockup | 4355 × 927.7, so **4.69 : 1** |
| Wordmark cap height | 430 (0.42 of the tile) |
| Font size | 591.1 (0.577 of the tile) |
| Gap, tile ink to name ink | 258 (0.60 cap heights) |
| Wordmark ink | 3169.2 wide |
| Baseline | the name's ink is centred on the tile's box centre |

The name is Inter SemiBold (`opsz` 32, `wght` 600), −1 % tracking, stored as outlines: **no font is needed
to open these files**, and the letters cannot reflow.

**Clear space.** Keep at least 232 units (a quarter of the tile) clear on every side. Nothing else goes in
that band: no second line, no tagline, no border.

**Smallest sizes.** Lockup 96 px wide; below that the name stops being readable, so use the mark alone. The
mark works down to 16 px (it is the favicon), but the eyes and the equalizer disappear under 32 px — that is
how the Oyenza tile behaves too.

## Colour

| | Hex | Where |
|---|---|---|
| Orange | `#F0892E` | The tile |
| Island | `#0A0A0A` | The island, hanging from the tile's top edge |
| Equalizer | `#FFA24A` | The four bars |
| Sunset | `#FFB37A` → `#F0708A` → `#2B4F86` → `#1B2F55` | The album cover, top to bottom |
| Ink | `#171717` | The name on light |
| White | `#FFFFFF` | The name on dark, and Oyen |

The name is `#171717` or `#FFFFFF`; never the orange, and never a gradient. The tile never changes colour,
never gets an outline, and never sits on a busy photograph without the clear space above.

## What not to do

- Don't set the name in another typeface, or re-space it. The tracking is part of the mark.
- Don't redraw the tile, the cat or the island in another style: they come from
  [`docs/brand`](../../docs/brand), where the rule is that Oyen's head is exactly the shape of the mark.
- Don't put the tile inside another shape, shadow it, or rotate it.
- Don't rebuild the lockup by placing `ariza-mark.png` next to live text: the gap and the baseline are
  measured, and live text drifts with the font.

## Redrawing

```sh
python3 -m pip install --user fonttools   # Pillow is already needed by tools/make_brand.py
python3 tools/make_ariza_brand.py
```

It writes every file in this folder and prints the measurements above, plus the ink gap between each pair
of letters, so a tracking change that collides two letters shows up as a number rather than as a look.

The wordmark needs Inter Variable (`Inter-VariableFont_opsz,wght.ttf`, SIL OFL) in `~/Library/Fonts`,
`/Library/Fonts` or this folder. SF Pro Text, which the site uses, cannot be redistributed as outlines.

These files are not covered by the MIT license that covers the code, like the rest of `docs/brand`.
