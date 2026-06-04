# Color Spaces

OCI uses OKLCH/OKLab as the canonical model because OKLab provides a practical
perceptual coordinate system for nearest-step distance calculations, while
OKLCH provides intuitive lightness, chroma, and hue coordinates for IDs.

sRGB, HEX, Display P3, Adobe RGB 1998, Rec.709, HSL, CSS, and JSON token forms
are representations of canonical OCI colors. They are not the identity model.

HEX is lossy 8-bit sRGB. CMYK numeric conversion is not implemented in v1-beta
and requires ICC profile support.
