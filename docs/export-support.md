# Export And Support Matrix

Exports are built from canonical OKLCH with:

```rust
export_all(color: Oklch) -> ExportSet
build_support_matrix(color: Oklch) -> SupportMatrix
```

## Target Systems

The core support matrix covers:

- sRGB float
- HEX
- RGB8
- HSL sRGB
- Display P3 float
- Adobe RGB 1998 float
- Rec.709 float
- OKLCH
- OKLab
- CSS
- JSON token
- CMYK placeholder

The CLI target names are:

```text
srgb
hex
rgb
hsl
display-p3
adobe-rgb
rec709
oklch
oklab
css
json-token
swift
tailwind
cmyk
```

## Support Status Enum

```text
supported
lossy
gamut_mapped
approximation
unsupported
profile_required
proof_required
user_supplied_reference
```

The enum contains future statuses, but the current implementation mostly emits:

- `supported`
- `lossy`
- `unsupported`
- `profile_required`

## RGB-Like Export Status

For sRGB, Display P3, Adobe RGB 1998, and Rec.709:

```text
canonical OKLCH
-> encoded target RGB
-> if all channels are finite and in [0, 1], status = supported
-> otherwise status = unsupported
```

When supported, round-trip error is:

```text
error = OKLabDistance(sourceOKLab, targetRoundTripOKLab)
```

Pretty CLI output also shows `ΔE CIEDE2000` on the next line. That value is
computed separately in CIELAB D65 using the CIEDE2000 formula.

where:

```text
OKLabDistance(A, B) = sqrt((LA - LB)^2 + (aA - aB)^2 + (bA - bB)^2)
```

## HEX And RGB8

HEX and RGB8 are sRGB-bound and 8-bit quantized.

If the color is outside sRGB gamut:

```text
status = unsupported
value = null
roundTripError = null
```

If the color is inside sRGB gamut:

```text
status = lossy
value = quantized 8-bit sRGB
roundTripError = OKLab distance after quantization round-trip
```

HEX is always uppercase:

```text
#E85A9A
```

## HSL

HSL is treated as an sRGB representation.

If the encoded sRGB color is in gamut:

```text
status = supported
value = HSL
roundTripError = OKLab distance after HSL -> sRGB -> OKLCH
```

If outside sRGB gamut:

```text
status = unsupported
```

## CSS

CSS export always includes OKLCH syntax:

```text
oklch(66.914300% 0.185968 355.308028deg)
```

CSS sRGB syntax is emitted only when sRGB is in gamut:

```text
rgb(232 90 154)
```

CSS Display P3 syntax is emitted only when Display P3 is in gamut:

```text
color(display-p3 0.844459 0.389513 0.597250)
```

CSS support status is `supported` with a note explaining that RGB syntax is
conditional on gamut.

## JSON Token

JSON-friendly export contains OKLCH and OKLab component structures. The CLI
serializes a compact selected JSON object; the core returns structs and leaves
serialization ownership to the caller.

## CMYK

CMYK numeric conversion is not implemented in v1-beta. The support entry is:

```text
status = profile_required
value = null
roundTripError = null
```

This is intentional. CMYK requires ICC/profile support and print workflow
decisions that are outside the current implementation.

## Out-Of-Gamut Policy

No gamut mapping is implemented in v1-beta. Out-of-gamut target exports are
marked `unsupported`, not approximated.
