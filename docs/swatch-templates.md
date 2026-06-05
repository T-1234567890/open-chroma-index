# SVG Swatch Templates

The OCI CLI can render SVG swatch cards from a text SVG template.

```text
oci swatch gen
oci swatch data
```

Generated swatch cards are template outputs for design, review, and
community/vendor production workflows. They are not official physical color
guarantees. Physical production requires proofing, material notes, printer
workflow, and measurement if used as production references.

## Template File

The expected template filename is:

```text
Color_Cards_OCI_v1.svg
```

Template lookup order:

1. `--template <SVG_PATH>`
2. `Color_Cards_OCI_v1.svg` in the current working directory
3. `templates/Color_Cards_OCI_v1.svg`
4. the bundled development template in the CLI crate

The renderer reads the SVG as UTF-8 text and replaces placeholder strings
exactly. It does not parse Illustrator-specific SVG structure.

## Color Block

The main color block should prefer OKLCH directly:

```svg
<rect id="COLOR_BLOCK" y="161.41" width="141.73" height="261.86" fill="oklch({{OKLCH}})"/>
```

`{{OKLCH}}` is replaced with CSS OKLCH components:

```text
69.586500% 0.149074 162.505253deg
```

If the template uses `fill="{{COLOR_CSS}}"`, the value is an `oklch(...)`
string. If it uses `fill="{{COLOR_HEX}}"`, the value is a clipped sRGB HEX
fallback for template compatibility.

For a precision OCI code with offset, the card uses the resolved precision
color. For a standard/base code, the card uses the registered base step color.

## Generate One Card

```bash
oci swatch gen --id OCI-1-22TL-326 --template Color_Cards_OCI_v1.svg --out out/
```

Default filename mode is `short`:

```text
out/OCI-1-22TL-326.svg
```

Use full filenames:

```bash
oci swatch gen --id OCI-1-22TL-326 --template Color_Cards_OCI_v1.svg --out out/ --filename full
```

```text
out/OCI-1-22TL-A3-L09-C06.svg
```

Existing output files are not overwritten unless `--overwrite` is passed.

## Generate A Family

```bash
oci swatch gen --family 22TL --template Color_Cards_OCI_v1.svg --out out/
oci swatch gen --family TL --template Color_Cards_OCI_v1.svg --out out/
oci swatch gen --family 22 --template Color_Cards_OCI_v1.svg --out out/
```

Family output goes into a family subdirectory:

```text
out/22TL/
  OCI-1-22TL-001.svg
  OCI-1-22TL-002.svg
  ...
  OCI-1-22TL-360.svg
```

## Generate A Range

Ranges are inclusive and must stay within one family in this version.

```bash
oci swatch gen --range OCI-1-22TL-001..OCI-1-22TL-120 --template Color_Cards_OCI_v1.svg --out out/
```

Compact range syntax is also supported:

```bash
oci swatch gen --range 22TL-001..22TL-120 --template Color_Cards_OCI_v1.svg --out out/
```

The CLI returns an error if the range crosses families or if the start step is
greater than the end step.

## Placeholder Data

Use `oci swatch data` to inspect the replacement map for one color:

```bash
oci swatch data --id OCI-1-22TL-326
```

The JSON object contains these keys:

```text
OCI_SHORT
OCI_FULL
FAMILY_INDEX
FAMILY_CODE
FAMILY_NAME
STEP_NUMBER
ANCHOR
LIGHTNESS_LEVEL
CHROMA_LEVEL
HEX
RGB
HSL
SRGB
DISPLAY_P3
ADOBE_RGB
REC709
OKLCH
OKLAB
COLOR_HEX
COLOR_CSS
VERSION
```

Recommended value shapes:

```text
HEX          #10B981
RGB          r=16 g=185 b=129
HSL          h=160.118201 s=0.840783 l=0.394121
SRGB         r=0.062751 g=0.725491 b=0.505883
DISPLAY_P3   r=0.332534 g=0.714700 b=0.522077
ADOBE_RGB    r=0.411523 g=0.719724 b=0.513062
REC709       r=0.023319 g=0.694674 b=0.456478
OKLCH        69.586500% 0.149074 162.505253deg
OKLAB        L=0.695865 a=-0.142179 b=0.044814
COLOR_HEX    #10B981
COLOR_CSS    oklch(69.586500% 0.149074 162.505253deg)
```

`FAMILY_CODE` is the four-character family id, such as `22TL`. In the bundled
template, the family line uses `{{FAMILY_NAME}}` followed by a tab and then
`{{FAMILY_CODE}}`.

Unavailable or out-of-gamut display fields use `Unavailable`.

Values are XML-escaped before insertion into SVG text.
