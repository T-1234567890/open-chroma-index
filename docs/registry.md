# Registry And Family Tables

The registry is frozen bundled data under `registry/v1/`.

Runtime loading uses `include_str!` for:

- `families.json`
- `steps.json`
- `test-vectors.json`
- `metadata.json`
- `checksums.json`
- `schema.json`

The runtime validates SHA-256 checksums before parsing families and steps. It
does not regenerate steps dynamically.

## Registry Invariants

```text
familyCount = 64
stepsPerFamily = 360
totalRegisteredBaseSteps = 23,040
hueAnchorsPerFamily = 3
lightnessLevels = 12
chromaLevels = 10
offsetPrecision = 6
canonicalModel = OKLCH/OKLab
familyModel = semantic-v1-beta
```

Validation checks:

- exactly 64 families
- exactly 23,040 steps
- exactly 360 steps per family
- unique family IDs
- unique step IDs and short IDs
- family index/code match
- step number matches anchor/lightness/chroma
- full and short step IDs match computed values

## Step Grid

Each family has:

```text
3 anchors * 12 lightness levels * 10 chroma levels = 360 steps
```

Step number:

```text
stepNumber = ((anchor - 1) * 120) + ((lightness - 1) * 10) + chroma
```

Reverse mapping:

```text
n = stepNumber - 1
anchor = floor(n/120) + 1
withinAnchor = n % 120
lightness = floor(withinAnchor/10) + 1
chroma = (withinAnchor % 10) + 1
```

Example:

```text
OCI-1-46PK-236 = OCI-1-46PK-A2-L12-C06
```

## Lightness Levels

| Level | L |
|---:|---:|
| L01 | 0.06 |
| L02 | 0.14 |
| L03 | 0.22 |
| L04 | 0.30 |
| L05 | 0.38 |
| L06 | 0.46 |
| L07 | 0.54 |
| L08 | 0.62 |
| L09 | 0.70 |
| L10 | 0.78 |
| L11 | 0.86 |
| L12 | 0.94 |

## Chroma Ratios

| Level | Ratio |
|---:|---:|
| C01 | 0.05 |
| C02 | 0.12 |
| C03 | 0.22 |
| C04 | 0.34 |
| C05 | 0.46 |
| C06 | 0.58 |
| C07 | 0.70 |
| C08 | 0.82 |
| C09 | 0.92 |
| C10 | 1.00 |

## Anchor Hue

For chromatic and earth/muted families:

```text
A1 = start + (end - start) * 1/6
A2 = start + (end - start) * 3/6
A3 = start + (end - start) * 5/6
```

For neutral families, all three anchors use a fixed hue from the generator:

```text
SL, SG: 230 degrees
GY, NG, WH, BK: 0 degrees
```

## Placeholder maxChroma

v1-beta uses deterministic placeholder chroma, not an ICC or gamut-search
max-chroma model.

Base chromatic max:

```text
lightnessWindow = max(1 - 0.75 * abs(2L - 1), 0.25)
chromaticMaxChroma = 0.40 * lightnessWindow
```

Family class:

```text
chromatic:    maxChroma = chromaticMaxChroma
earth_muted:  maxChroma = chromaticMaxChroma * 0.55
neutral:      maxChroma = fixed neutral max
```

Step chroma:

```text
C = maxChroma * chromaRatio
```

Neutral fixed max chroma:

| Family | Max chroma |
|---|---:|
| 58SL | 0.035 |
| 59SG | 0.018 |
| 60GY | 0.012 |
| 61NG | 0.006 |
| 62WH | 0.004 |
| 63BK | 0.004 |

## Family Classes

- `chromatic`: normal hue-based families.
- `earth_muted`: semantic muted families selected by lightness, chroma, and hue.
- `neutral`: very low chroma families selected by chroma and lightness.

Earth/muted and neutral families do not own ordinary equal-width hue sectors in
the encoder. The `hueStart` and `hueEnd` fields are still stored in
`families.json` for frozen step placement and metadata.

## Full 64-Family Table

| Index | ID | Code | Name | Group | Class | Hue start | Hue end |
|---:|---|---|---|---|---|---:|---:|
| 00 | 00RD | RD | Red | Reds to Yellows | chromatic | 0.000000 | 8.000000 |
| 01 | 01VR | VR | Vermilion Red | Reds to Yellows | chromatic | 8.000000 | 15.090909 |
| 02 | 02VM | VM | Vermilion | Reds to Yellows | chromatic | 15.090909 | 22.181818 |
| 03 | 03CR | CR | Coral Red | Reds to Yellows | chromatic | 22.181818 | 29.272727 |
| 04 | 04CO | CO | Coral | Reds to Yellows | chromatic | 29.272727 | 36.363636 |
| 05 | 05PO | PO | Peach Orange | Reds to Yellows | chromatic | 36.363636 | 43.454545 |
| 06 | 06OR | OR | Orange | Reds to Yellows | chromatic | 43.454545 | 50.545455 |
| 07 | 07AO | AO | Amber Orange | Reds to Yellows | chromatic | 50.545455 | 57.636364 |
| 08 | 08AM | AM | Amber | Reds to Yellows | chromatic | 57.636364 | 64.727273 |
| 09 | 09GD | GD | Gold | Reds to Yellows | chromatic | 64.727273 | 71.818182 |
| 10 | 10YW | YW | Yellow | Reds to Yellows | chromatic | 71.818182 | 78.909091 |
| 11 | 11LY | LY | Lemon Yellow | Reds to Yellows | chromatic | 78.909091 | 86.000000 |
| 12 | 12LM | LM | Lime | Lime to Teals | chromatic | 86.000000 | 93.090909 |
| 13 | 13LG | LG | Lime Green | Lime to Teals | chromatic | 93.090909 | 100.181818 |
| 14 | 14YG | YG | Yellow Green | Lime to Teals | chromatic | 100.181818 | 107.272727 |
| 15 | 15GR | GR | Green | Lime to Teals | chromatic | 107.272727 | 114.363636 |
| 16 | 16FG | FG | Forest Green | Lime to Teals | chromatic | 114.363636 | 121.454545 |
| 17 | 17EG | EG | Emerald Green | Lime to Teals | chromatic | 121.454545 | 128.545455 |
| 18 | 18EM | EM | Emerald | Lime to Teals | chromatic | 128.545455 | 135.636364 |
| 19 | 19MN | MN | Mint Green | Lime to Teals | chromatic | 135.636364 | 142.727273 |
| 20 | 20MT | MT | Mint | Lime to Teals | chromatic | 142.727273 | 149.818182 |
| 21 | 21SE | SE | Sea Green | Lime to Teals | chromatic | 149.818182 | 156.909091 |
| 22 | 22TL | TL | Teal | Lime to Teals | chromatic | 156.909091 | 164.000000 |
| 23 | 23CT | CT | Cyan Teal | Lime to Teals | chromatic | 164.000000 | 171.090909 |
| 24 | 24CY | CY | Cyan | Cyans to Purples | chromatic | 171.090909 | 178.181818 |
| 25 | 25AQ | AQ | Aqua | Cyans to Purples | chromatic | 178.181818 | 185.272727 |
| 26 | 26AZ | AZ | Azure | Cyans to Purples | chromatic | 185.272727 | 192.363636 |
| 27 | 27SB | SB | Sky Blue | Cyans to Purples | chromatic | 192.363636 | 199.454545 |
| 28 | 28SK | SK | Sky | Cyans to Purples | chromatic | 199.454545 | 206.545455 |
| 29 | 29LB | LB | Light Blue | Cyans to Purples | chromatic | 206.545455 | 213.636364 |
| 30 | 30BL | BL | Blue | Cyans to Purples | chromatic | 213.636364 | 220.727273 |
| 31 | 31RB | RB | Royal Blue | Cyans to Purples | chromatic | 220.727273 | 227.818182 |
| 32 | 32CB | CB | Cobalt Blue | Cyans to Purples | chromatic | 227.818182 | 234.909091 |
| 33 | 33NV | NV | Navy | Cyans to Purples | chromatic | 234.909091 | 242.000000 |
| 34 | 34IB | IB | Indigo Blue | Cyans to Purples | chromatic | 242.000000 | 249.090909 |
| 35 | 35IN | IN | Indigo | Cyans to Purples | chromatic | 249.090909 | 256.181818 |
| 36 | 36IV | IV | Indigo Violet | Cyans to Purples | chromatic | 256.181818 | 263.272727 |
| 37 | 37VT | VT | Violet | Cyans to Purples | chromatic | 263.272727 | 270.363636 |
| 38 | 38LV | LV | Lavender Violet | Lavenders to Pinks | chromatic | 270.363636 | 277.454545 |
| 39 | 39LA | LA | Lavender | Lavenders to Pinks | chromatic | 277.454545 | 284.545455 |
| 40 | 40PR | PR | Purple | Lavenders to Pinks | chromatic | 284.545455 | 291.636364 |
| 41 | 41BP | BP | Blue Purple | Lavenders to Pinks | chromatic | 291.636364 | 298.727273 |
| 42 | 42MA | MA | Magenta | Lavenders to Pinks | chromatic | 298.727273 | 305.818182 |
| 43 | 43FM | FM | Fuchsia Magenta | Lavenders to Pinks | chromatic | 305.818182 | 312.909091 |
| 44 | 44FS | FS | Fuchsia | Lavenders to Pinks | chromatic | 312.909091 | 320.000000 |
| 45 | 45HP | HP | Hot Pink | Lavenders to Pinks | chromatic | 320.000000 | 330.000000 |
| 46 | 46PK | PK | Pink | Lavenders to Pinks | chromatic | 330.000000 | 340.000000 |
| 47 | 47RP | RP | Rose Pink | Lavenders to Pinks | chromatic | 340.000000 | 348.000000 |
| 48 | 48RS | RS | Rose | Lavenders to Pinks | chromatic | 348.000000 | 356.000000 |
| 49 | 49WR | WR | Wine Red | Lavenders to Pinks | chromatic | 356.000000 | 360.000000 |
| 50 | 50MR | MR | Maroon Red | Lavenders to Pinks | earth_muted | 340.000000 | 352.000000 |
| 51 | 51MU | MU | Maroon | Lavenders to Pinks | earth_muted | 300.000000 | 320.000000 |
| 52 | 52BR | BR | Brown | Earth & Muted Tones | earth_muted | 20.000000 | 55.000000 |
| 53 | 53CP | CP | Copper Brown | Earth & Muted Tones | earth_muted | 18.000000 | 48.000000 |
| 54 | 54TN | TN | Tan | Earth & Muted Tones | earth_muted | 35.000000 | 70.000000 |
| 55 | 55BG | BG | Beige | Earth & Muted Tones | earth_muted | 45.000000 | 85.000000 |
| 56 | 56OL | OL | Olive | Earth & Muted Tones | earth_muted | 65.000000 | 105.000000 |
| 57 | 57OG | OG | Olive Green | Earth & Muted Tones | earth_muted | 90.000000 | 135.000000 |
| 58 | 58SL | SL | Slate | Grayscale & Neutrals | neutral | 0.000000 | 360.000000 |
| 59 | 59SG | SG | Slate Gray | Grayscale & Neutrals | neutral | 0.000000 | 360.000000 |
| 60 | 60GY | GY | Gray | Grayscale & Neutrals | neutral | 0.000000 | 360.000000 |
| 61 | 61NG | NG | Neutral Gray | Grayscale & Neutrals | neutral | 0.000000 | 360.000000 |
| 62 | 62WH | WH | White Neutral | Grayscale & Neutrals | neutral | 0.000000 | 360.000000 |
| 63 | 63BK | BK | Black Neutral | Grayscale & Neutrals | neutral | 0.000000 | 360.000000 |
