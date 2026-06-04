# CLI Commands

The CLI package is `oci-cli`; the binary name is `oci`.

The CLI uses a small internal parser. Arguments are positional plus long flags.
Known flags that take values are:

```text
--space --format --precision --exports --to --from --type --path
```

Default output comes from configuration. Built-in defaults use `pretty`. JSON is
always available with `--format json`.

## Command Summary

```text
oci encode <INPUT> --space <SPACE> [--format json|pretty|plain] [--precision <N>] [--no-exports] [--path <TOML_PATH>]
oci inspect <OCI_ID> [--format json|pretty|plain] [--exports all|none|summary|<LIST>] [--precision <N>] [--path <TOML_PATH>]
oci export <OCI_ID> [--to <TARGETS>] [--format json|plain|pretty] [--path <TOML_PATH>]
oci convert <INPUT> [--from <SPACE>] [--to <TARGETS>] [--format json|plain|pretty] [--precision <N>] [--path <TOML_PATH>]
oci registry <SUBCOMMAND> [--path <TOML_PATH>]
oci test <SUBCOMMAND> [--path <TOML_PATH>]
oci validate <TARGET> [--type id|registry|color] [--space <SPACE>] [--path <TOML_PATH>]
oci config [--path <TOML_PATH>]
```

## Supported Input Spaces

`encode`, `convert`, and `validate --type color` support:

```text
hex
rgb
hsl
srgb
display-p3
adobe-rgb
rec709
oklch
oklab
oci
```

Input parsing rules:

- `hex`: `#RRGGBB`, `RRGGBB`, `#RGB`, or `RGB`.
- `rgb`: three 8-bit integer components, split by comma, slash, or spaces.
- `srgb`, `display-p3`, `adobe-rgb`, `rec709`: three encoded float components.
- `hsl`: hue in degrees plus saturation and lightness floats in `0.0..=1.0`.
- `oklch`: `L C H` as floats.
- `oklab`: `L a b` as floats.
- `oci`: short or full OCI ID with optional offset.

Component strings are split on commas, slashes, or spaces:

```text
232,90,154
232 90 154
232/90/154
```

## Supported Export Targets

```text
hex
rgb
hsl
srgb
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

`swift` and `tailwind` are CLI-only generated string snippets. CMYK never emits
numeric CMYK in v1-beta; it returns `profile_required`.

## `oci encode`

Purpose: convert an input color to canonical OKLCH, find the nearest registered
base step, calculate offset, and print an OCI identity.

```text
oci encode <INPUT> --space <SPACE> [--format json|pretty|plain] [--precision <N>] [--no-exports]
```

Examples:

```text
oci encode "#E85A9A" --space hex
oci encode "#E85A9A" --space hex --format json
oci encode "232,90,154" --space rgb --precision 4
oci encode "0.669143 0.185968 355.308028" --space oklch
```

Pretty output includes:

- input and source space
- `OCI standard color code`, the registered base code without offset
- `OCI precision color code`, the code with OKLCH offset when an offset exists
- canonical OKLCH
- configured default exports unless hidden
- support matrix count unless hidden
- warnings line unless hidden

Plain output returns only the configured preferred OCI code.

JSON output includes:

- `input`
- `sourceSpace`
- `canonical.oklch`
- `canonical.oklab`
- `oci.short`
- `oci.full`
- `oci.precisionShort`
- `oci.precisionFull`
- `offset`
- `exports`
- `support`
- `warnings`

`--no-exports` suppresses `exports` and `support` in JSON and pretty output.

## `oci inspect`

Purpose: parse an OCI ID, decode the registered base step plus offset, and show
canonical color plus exports.

```text
oci inspect <OCI_ID> [--format json|pretty|plain] [--exports all|none|summary|<LIST>] [--precision <N>]
```

Examples:

```text
oci inspect OCI-1-46PK-236
oci inspect OCI-1-46PK-A2-L12-C06 --format json
oci inspect OCI-1-48RS-327@L-0.030857,C-0.010032,H+0.641361 --exports hex,oklch,css
```

`--exports` modes:

- `none`: do not print exports.
- `summary`: use `inspect.default_export_list` from config.
- `list`: same behavior as `summary` in the current implementation.
- `all`: all known export targets.
- comma-separated list: selected export targets.

Plain output returns the canonical short OCI ID.

## `oci export`

Purpose: decode an OCI ID and export selected target representations.

```text
oci export <OCI_ID> [--to <TARGETS>] [--format json|plain|pretty]
```

Examples:

```text
oci export OCI-1-46PK-236 --to hex,oklch,css
oci export OCI-1-46PK-236 --to cmyk --format json
```

If `--to` is omitted, the command uses `output.default_exports` from config.

## `oci convert`

Purpose: convert directly from an input color space to selected target exports.
It still uses the OCI encode pipeline internally so it can report canonical
OKLCH and OCI identity, but the command's primary purpose is representation
conversion.

```text
oci convert <INPUT> [--from <SPACE>] [--to <TARGETS>] [--format json|plain|pretty] [--precision <N>]
```

Examples:

```text
oci convert "#E85A9A" --from hex --to srgb,display-p3,oklch
oci convert "0.91 0.35 0.61" --from srgb --to hex,css
```

If `--from` is omitted, it uses `color.default_input_space`. If `--to` is
omitted, it uses `color.default_targets`.

## `oci registry`

Purpose: inspect and validate the frozen bundled registry.

```text
oci registry info
oci registry families
oci registry family <INDEX_OR_CODE_OR_ID>
oci registry step <OCI_ID_OR_STEP_ID>
oci registry validate
oci registry checksum
```

Behavior:

- `info`: JSON object with version, family count, and step count.
- `families`: JSON list of all loaded families.
- `family`: JSON detail for one family, including step count.
- `step`: JSON detail for one registered base step.
- `validate`: validates family count, total step count, per-family step count,
  unique step IDs, and ID mappings.
- `checksum`: calculates SHA-256 of bundled frozen data and compares to
  `checksums.json`.

The current CLI always emits JSON for registry commands.

## `oci test`

Purpose: run built-in compatibility checks.

```text
oci test vectors
oci test roundtrip
oci test registry
```

Behavior:

- `vectors`: runs entries from bundled `registry/v1/test-vectors.json`.
- `roundtrip`: encodes `#E85A9A`, decodes it, and re-encodes it.
- `registry`: validates registry invariants.

The current CLI emits JSON for test commands.

## `oci validate`

Purpose: validate an OCI ID, registry, or color input.

```text
oci validate <TARGET> [--type id|registry|color] [--space <SPACE>]
```

Examples:

```text
oci validate OCI-1-46PK-236 --type id
oci validate registry --type registry
oci validate "#E85A9A" --type color --space hex
```

Default `--type` is `id`. For `--type color`, default `--space` is `hex`.

## `oci config`

Purpose: create or edit TOML configuration.

```text
oci config
oci config --path /custom/path/config.toml
```

When stdin is a terminal, the binary opens an interactive wizard. When used
through tests or non-interactive process execution, it writes current or
built-in defaults without prompting.

See [Configuration](configuration.md).

## Error Codes

CLI errors are printed as JSON on stderr:

```json
{"error":{"code":"parse_error","message":"..."}}
```

Current structured codes:

- `parse_error`
- `invalid_family`
- `invalid_step`
- `invalid_offset`
- `invalid_id`
- `unsupported_space`
- `registry_error`
- `config_error`
