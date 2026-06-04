# OCI ID Format

OCI IDs have short and full forms. Both identify the same registered base step.
Both forms can include an optional OKLCH offset.

## Version

The implemented version is:

```text
1
```

All IDs must start with:

```text
OCI-1-
```

## Family ID

Family ID format:

```text
NNCODE
```

Where:

- `NN` is a two-digit family index.
- `CODE` is two uppercase ASCII letters.

Example:

```text
46PK
```

The parser validates that the index and code match the frozen registry. For
example, `46BK` fails because family index `46` is `PK`, not `BK`.

## Short Base Form

```text
OCI-1-46PK-236
```

Parts:

```text
OCI
1
46PK
236
```

The last part is a three-digit step number in `001..=360`.

## Full Base Form

```text
OCI-1-46PK-A2-L12-C06
```

Parts:

```text
OCI
1
46PK
A2
L12
C06
```

Component ranges:

```text
anchor:     A1..A3
lightness:  L01..L12
chroma:     C01..C10
```

## Short/Full Mapping

Short step number from full components:

```text
stepNumber = ((anchor - 1) * 120) + ((lightness - 1) * 10) + chroma
```

Full components from short step number:

```text
n = stepNumber - 1
anchor = floor(n/120) + 1
withinAnchor = n % 120
lightness = floor(withinAnchor/10) + 1
chroma = (withinAnchor % 10) + 1
```

Example:

```text
OCI-1-46PK-236
OCI-1-46PK-A2-L12-C06
```

Mapping:

```text
n = 236 - 1 = 235
anchor = floor(235/120) + 1 = 2
withinAnchor = 235 % 120 = 115
lightness = floor(115/10) + 1 = 12
chroma = (115 % 10) + 1 = 6
```

## Precision Offset

Offset syntax:

```text
@L+0.002134,C-0.001042,H+0.218400
```

Full example:

```text
OCI-1-46PK-236@L+0.002134,C-0.001042,H+0.218400
OCI-1-46PK-A2-L12-C06@L+0.002134,C-0.001042,H+0.218400
```

Offset components are signed decimal numbers:

```text
L{sign}{value},C{sign}{value},H{sign}{value}
```

The formatter always emits six decimal places.

## Offset Meaning

The offset is the residual from the registered base step to the canonical input
OKLCH:

```text
offsetL = inputL - baseL
offsetC = inputC - baseC
offsetH = shortestHueDiff(inputH, baseH)
```

Hue uses shortest signed circular difference:

```text
diff = ((targetHue - baseHue + 180) mod 360) - 180
if diff == -180:
  diff = 180
```

## Decoding

Decode applies:

```text
decodedL = baseL + offsetL
decodedC = baseC + offsetC
decodedH = baseH + offsetH
decodedH = decodedH mod 360
```

If no offset is present, all offset components are treated as zero.

## Validation Failures

The parser rejects:

- missing `OCI` prefix
- unsupported version
- malformed family ID
- non-uppercase family code
- unknown family
- family index/code mismatch
- short step outside `001..=360`
- full anchor outside `A1..A3`
- full lightness outside `L01..L12`
- full chroma outside `C01..C10`
- malformed offset
- more than one `@`
