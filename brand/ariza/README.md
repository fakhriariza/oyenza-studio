# Ariza Studio brand files

Ariza Studio in the Oyenza Studio system: the same orange tile, the island hanging from its top edge playing
music (album cover and equalizer, as on the real island). Where Oyen sat, Ariza has its own mark — an **"A"**
monogram, Inter's Bold letter, centred in the field the island leaves below itself. The name reads
**Ariza Studio**, set the way the site sets *Oyenza Studio*: semibold, −0.01em tracking, ink `#171717`.

The island, the album cover and the equalizer come from [`tools/make_brand.py`](../../tools/make_brand.py),
unchanged. [`tools/make_ariza_brand.py`](../../tools/make_ariza_brand.py) takes its tile, paints Oyen out of
it, puts the monogram in his place, and adds the wordmark.

| File | What |
|---|---|
| `ariza-mark.svg`, `ariza-mark-{16,32,64,128,256,512,1024}.png` | The tile alone: favicon, app icon, avatar, tray |
| `ariza-studio-icon.ico` | The same tile as a Windows icon, every size drawn on its own (16–256) |
| `ariza-studio-lockup.svg`, `.png`, `-1024.png` | Tile and wordmark, for light backgrounds |
| `ariza-studio-lockup-dark.svg`, `.png` | The same for dark backgrounds; the tile is unchanged, the name turns white |
| `preview.png` | The sheet: both lockups, the mark at 200/128/64/32 px, the palette |

The PNGs are drawn from the same geometry as the SVGs, three times over and then scaled down, so the two
never disagree. They are 2048 and 1024 pixels wide.

## The mark

Measured in the tile's own units (its box is 1024 square, its visible ink 927.7):

| | |
|---|---|
| Letter | `A`, Inter Bold (`wght` 700), stored as outlines |
| Cap height | 440.7 — 0.475 of the tile's visible side |
| Ink | 426.7 wide, so almost square (0.968) |
| Centre | 608.9 from the tile's top edge, the middle of the 733.9 the island leaves under itself |
| Air | 146.5 above the apex, 146.6 under the feet |

The monogram is a step bolder than the name on purpose: at 32 px a SemiBold "A" thins out, and a mark has
to hold at the size a tray icon is drawn.

## The lockup

| | |
|---|---|
| Lockup | 4355 × 927.7, so **4.69 : 1** |
| Wordmark cap height | 430 (0.42 of the tile) |
| Font size | 591.1 (0.577 of the tile) |
| Gap, tile ink to name ink | 258 (0.60 cap heights) |
| Wordmark ink | 3169.2 wide |
| Baseline | the name's ink is centred on the tile's box centre |

The name is Inter SemiBold (`wght` 600), −1 % tracking, stored as outlines: **no font is needed to open
these files**, and the letters cannot reflow.

**Clear space.** Keep at least 232 units (a quarter of the tile) clear on every side. Nothing else goes in
that band: no second line, no tagline, no border.

**Smallest sizes.** Lockup 96 px wide; below that the name stops being readable, so use the mark alone. The
mark works down to 16 px (it is the favicon), but the island's cover and the equalizer disappear under 32 px
— that is how the Oyenza tile behaves too.

## Colour

| | Hex | Where |
|---|---|---|
| Orange | `#F0892E` | The tile |
| Island | `#0A0A0A` | The island, hanging from the tile's top edge |
| Equalizer | `#FFA24A` | The four bars |
| Sunset | `#FFB37A` → `#F0708A` → `#2B4F86` → `#1B2F55` | The album cover, top to bottom |
| Ink | `#171717` | The name on light |
| White | `#FFFFFF` | The name on dark, and the monogram |

The name is `#171717` or `#FFFFFF`; never the orange, and never a gradient. The tile never changes colour,
never gets an outline, and never sits on a busy photograph without the clear space above.

## What not to do

- Don't set the name in another typeface, or re-space it. The tracking is part of the mark.
- Don't replace the "A" with another letterform, another weight, or a drawing: it is Inter's own outline,
  kept so the mark and the name are cut from one typeface.
- Don't move the monogram or resize it inside the tile: it is centred in the field under the island, and
  anything else makes the tile top-heavy.
- Don't redraw the island, the cover or the bars. They are Oyenza's, and they are what makes this read as
  the same family.
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

Both cuts need Inter Variable (`Inter-VariableFont_opsz,wght.ttf`, SIL OFL) in `~/Library/Fonts`,
`/Library/Fonts` or this folder. SF Pro Text, which the site uses, cannot be redistributed as outlines.

These files are not covered by the MIT license that covers the code, like the rest of `docs/brand`.
