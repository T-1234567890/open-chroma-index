# OCI Identity Algorithm

This page describes the implemented v1-beta identity pipeline in `src/index.rs`.

## High-Level Encode Pipeline

```text
input color
-> parse source
-> convert to canonical OKLCH
-> select candidate families
-> find nearest registered base step by OKLab Euclidean distance
-> calculate OKLCH offset
-> format short/full OCI ID
-> decode ID back to OKLCH
-> build exports and support matrix
```

Public entry points:

```rust
encode(input: ColorInput, registry: &Registry) -> Result<EncodeResult, OciPipelineError>
encode_from_hex(hex: &str, registry: &Registry) -> Result<EncodeResult, OciPipelineError>
```

## Canonicalization

All input forms become `Oklch`:

- `Hex` -> `EncodedSrgb::from_hex` -> OKLCH
- `Srgb`/`SrgbRgb` -> encoded sRGB -> OKLCH
- `HslSrgb` -> encoded sRGB -> OKLCH
- `DisplayP3Float` -> encoded Display P3 -> OKLCH
- `AdobeRgb1998Float` -> encoded Adobe RGB 1998 -> OKLCH
- `Rec709Float` -> encoded Rec.709 -> OKLCH
- `Oklab` -> OKLCH
- `Oklch` -> normalized OKLCH
- `OciId`/`OciIdString` -> decode ID -> OKLCH

OKLCH normalization:

```text
if C < 0:
  C = -C
  H = H + 180

if abs(C) < 1e-12:
  C = 0
  H = 0

if any component is non-finite:
  replace it with 0

H = H mod 360
```

## Candidate Family Selection

The encoder does not search all 64 families. It searches a semantic candidate
set chosen from canonical OKLCH.

Constants:

```text
NEUTRAL_CHROMA_LIMIT = 0.035
MUTED_CHROMA_LIMIT = 0.17
```

Selection order:

```text
if C <= 0.035:
  use neutral_candidate_family_indices
else if muted_candidate_family_indices returns a set:
  use that set
else:
  use chromatic_candidate_family_indices(H)
```

### Neutral Candidates

```text
if L <= 0.18:
  [63]                  # BK
else if L >= 0.86:
  [62]                  # WH
else if H in [190, 270) and C > 0.012:
  [58, 59, 60, 61]      # SL, SG, GY, NG
else if C <= 0.006:
  [61, 60, 59]          # NG, GY, SG
else if C <= 0.012:
  [60, 61, 59]          # GY, NG, SG
else:
  [60, 59, 58, 61]      # GY, SG, SL, NG
```

High-chroma colors cannot reach this branch, so they cannot encode as `BK`,
`WH`, `GY`, `NG`, `SG`, or `SL`.

### Earth/Muted Candidates

```text
if C <= 0.17 and L <= 0.52 and (H in [335, 360) or H in [0, 15)):
  [50, 49, 48]          # MR, WR, RS

if C <= 0.17 and L <= 0.55 and H in [15, 55):
  [52, 53, 54]          # BR, CP, TN

if C <= 0.14 and L > 0.50 and H in [35, 90):
  [54, 55, 52]          # TN, BG, BR

if C <= 0.17 and H in [65, 135):
  [56, 57]              # OL, OG

if C <= 0.06 and L in [0.22, 0.68] and H in [190, 270):
  [58, 59]              # SL, SG
```

### Chromatic Candidates

Chromatic families are indices `00..49`. The current family is the chromatic
family whose hue range contains `H`. Candidate set:

```text
[current, previous, next]
```

Wraparound:

```text
previous(00RD) = 49WR
next(49WR) = 00RD
```

Chromatic ranges:

```text
00RD: [0, 8)
01..44: start = 8 + (index - 1) * (312/44)
        end   = 8 + index * (312/44)
45HP: [320, 330)
46PK: [330, 340)
47RP: [340, 348)
48RS: [348, 356)
49WR: [356, 360)
```

## Nearest Registered Step

For each candidate family, the encoder checks that family's 360 frozen steps.
Each base step is converted to OKLab and compared with the input OKLab.

Distance:

```text
d = sqrt((L1 - L2)^2 + (a1 - a2)^2 + (b1 - b2)^2)
```

Tie epsilon:

```text
TIE_EPSILON = 1e-12
```

Tie-break order:

```text
1. smaller distance
2. lower family index
3. lower anchor
4. lower lightness level
5. lower chroma level
```

## Offset Calculation

The selected base step has OKLCH:

```text
base = (Lb, Cb, Hb)
input = (Li, Ci, Hi)
```

Offset:

```text
offsetL = Li - Lb
offsetC = Ci - Cb
offsetH = shortestHueDiff(Hi, Hb)
```

Hue difference:

```text
diff = ((targetHue - baseHue + 180) mod 360) - 180
if diff == -180:
  diff = 180
```

Offsets are rounded to six decimal places:

```text
round(value * 1_000_000)/1_000_000
```

Values smaller than half a unit at this precision become `0.0`.

If all offset components are zero, the OCI ID is emitted without an offset.

## Decode Pipeline

```text
OCI ID
-> parse family and step
-> validate against frozen registry
-> find registered base step
-> base OKLCH
-> add optional offset
-> normalize OKLCH
```

Public entry point:

```rust
decode_oci_id(id: &OciId, registry: &Registry) -> Result<Oklch, OciPipelineError>
```

## Canonical ID

Canonicalization is implemented as:

```text
decode OCI ID -> OKLCH -> encode OKLCH -> canonical OCI ID
```

Public entry point:

```rust
canonicalize_oci_id(id: &OciId, registry: &Registry) -> Result<OciId, OciPipelineError>
```

## Inspect Pipeline

`inspect` parses/decodes an ID and returns:

- input ID
- short ID
- full ID
- canonical OKLCH
- canonical short/full ID after decode/encode
- exports
- support matrix

Public entry point:

```rust
inspect(id: &OciId, registry: &Registry) -> Result<InspectResult, OciPipelineError>
```
