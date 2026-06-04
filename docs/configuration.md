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
custom config path passed with --path or --config
installed CLI config at <oci-install-dir>/config.toml
built-in defaults
```

Examples:

```text
oci encode "#E85A9A" --space hex
oci encode "#E85A9A" --space hex --path /tmp/oci.toml
oci encode "#E85A9A" --space hex --path /tmp/oci.toml --format json
oci serve --config /tmp/oci.toml --port 9000
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
- Local Kernel API server host
- Local Kernel API server port
- whether to warn when the server is not bound to localhost

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
verify = false
default_exports = ["hex", "rgb", "hsl", "srgb", "display-p3", "adobe-rgb", "rec709", "oklch", "oklab", "css", "json-token", "swift", "tailwind", "cmyk"]

[encode]
include_offset = true
prefer_short_code = true
include_full_code = false

[inspect]
exports = "summary"
default_export_list = ["hex", "rgb", "hsl", "srgb", "display-p3", "adobe-rgb", "rec709", "oklch", "oklab", "css", "json-token", "swift", "tailwind", "cmyk"]

[registry]
version = "v1"
source = "bundled"
path = ""
validate_on_start = false

[color]
default_input_space = "hex"
default_targets = ["hex", "oklch", "display-p3"]

[server]
host = "127.0.0.1"
port = 8765
warn_non_localhost = true
```

## Field Reference

### `[output]`

- `format`: `pretty`, `json`, or `plain`.
- `precision`: number of decimal places for pretty output.
- `show_support`: include support matrix summary in pretty output.
- `show_warnings`: include warnings line in pretty output.
- `show_exports`: include export section in pretty output and JSON encode
  output unless `--no-exports` is used.
- `verify`: include detailed per-target verification lines in pretty output.
  Pretty output still includes the compact `verification:` block when exports
  are visible.
- `default_exports`: export targets used by `export` when `--to` is omitted.
  Built-in defaults include every supported CLI export target.

### `[encode]`

- `include_offset`: if true, the preferred code includes precision offset when
  nonzero.
- `prefer_short_code`: if true, pretty/plain encode prefer short IDs.
- `include_full_code`: if true, pretty encode also prints full ID.

### `[inspect]`

- `exports`: default inspect export mode: `all`, `none`, `summary`, `list`, or a
  comma-separated list supplied with `--exports`.
- `default_export_list`: stored config value for inspect summary/list behavior.
  Built-in CLI behavior currently shows every supported CLI export target for
  inspect `summary`, `list`, and `all`.

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

### `[server]`

- `host`: default host for `oci serve`. The built-in default is `127.0.0.1`.
- `port`: default port for `oci serve`. The built-in default is `8765`.
- `warn_non_localhost`: if true, `oci serve` prints a warning when bound to a
  host other than `127.0.0.1`, `localhost`, or `::1`.

CLI flags always override server config:

```text
oci serve --host 127.0.0.1 --port 9000
oci serve --config /tmp/oci.toml --port 9000
```

The Local Kernel API always returns JSON. CLI output defaults such as
`output.format = "pretty"` do not change API response envelopes.

## TOML Parser Notes

The current parser is intentionally small and supports the implemented TOML
shape: section headers, quoted strings, booleans, unsigned integers, and arrays
of quoted strings. Unknown sections or keys return `config_error`. This is not
a general-purpose TOML parser.
