# CLI Configuration

The CLI configuration format is TOML. The configuration file is not stored in
the repository. By default it is placed beside the installed `oci` executable:

```text
<oci-install-dir>/config.toml
```

The implementation resolves this path with `std::env::current_exe()`, then
joins `config.toml` to the executable's parent directory. If that lookup fails,
it falls back to a relative `config.toml`.

## Path Priority

Runtime config priority:

```text
CLI flags
custom config path passed with --path
installed CLI config at <oci-install-dir>/config.toml
built-in defaults
```

Examples:

```text
oci encode "#E85A9A" --space hex
oci encode "#E85A9A" --space hex --path /tmp/oci.toml
oci encode "#E85A9A" --space hex --path /tmp/oci.toml --format json
```

In the last example, `--format json` overrides `output.format` from the TOML
file.

## Creating Config

```text
oci config
oci config --path /custom/path/config.toml
```

If the selected file does not exist, `oci config` creates it.

In an interactive terminal, the command prompts for:

- output format
- precision
- default export targets
- whether to show support matrix
- whether to show warnings
- whether to show exports
- whether to include offset
- whether to prefer short code
- whether to include full code
- default inspect export behavior
- inspect export list
- default input color space
- default convert targets
- registry source
- registry path
- registry validation behavior

In non-interactive execution, the command writes defaults without prompting.

## Built-In Defaults

If no config file exists, the CLI uses:

```toml
[output]
format = "pretty"
precision = 6
show_support = true
show_warnings = true
show_exports = true
default_exports = ["hex", "oklch", "display-p3", "css"]

[encode]
include_offset = true
prefer_short_code = true
include_full_code = false

[inspect]
exports = "summary"
default_export_list = ["hex", "oklch", "srgb", "display-p3"]

[registry]
version = "v1"
source = "bundled"
path = ""
validate_on_start = false

[color]
default_input_space = "hex"
default_targets = ["hex", "oklch", "display-p3"]
```

## Field Reference

### `[output]`

- `format`: `pretty`, `json`, or `plain`.
- `precision`: number of decimal places for pretty output.
- `show_support`: include support matrix summary in pretty output.
- `show_warnings`: include warnings line in pretty output.
- `show_exports`: include export section in pretty output and JSON encode
  output unless `--no-exports` is used.
- `default_exports`: export targets used by `encode` pretty output and by
  `export` when `--to` is omitted.

### `[encode]`

- `include_offset`: if true, the preferred code includes precision offset when
  nonzero.
- `prefer_short_code`: if true, pretty/plain encode prefer short IDs.
- `include_full_code`: if true, pretty encode also prints full ID.

### `[inspect]`

- `exports`: default inspect export mode: `all`, `none`, `summary`, `list`, or a
  comma-separated list supplied with `--exports`.
- `default_export_list`: targets used when inspect export mode is `summary` or
  `list`.

### `[registry]`

- `version`: currently `v1`.
- `source`: currently only `bundled` is supported at runtime.
- `path`: reserved for future external registry loading.
- `validate_on_start`: if true, validates the bundled registry before command
  execution.

If `source` is not `bundled`, commands return `registry_error`.

### `[color]`

- `default_input_space`: used by `encode` and `convert` when the command does
  not provide `--space` or `--from`.
- `default_targets`: used by `convert` when `--to` is omitted.

## TOML Parser Notes

The current parser is intentionally small and supports the implemented TOML
shape: section headers, quoted strings, booleans, unsigned integers, and arrays
of quoted strings. Unknown sections or keys return `config_error`. This is not
a general-purpose TOML parser.
