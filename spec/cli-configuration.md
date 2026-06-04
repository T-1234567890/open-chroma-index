# CLI Configuration

The `oci` CLI supports TOML configuration for developer-facing defaults. The
configuration file changes output behavior and default command options, but CLI
flags always win over configuration values.

## Path Selection

Default path:

```text
<oci-install-dir>/config.toml
```

Supported forms:

```text
oci config
oci config --path /custom/path/config.toml
```

Other CLI commands also accept `--path`:

```text
oci inspect OCI-1-46PK-236 --path /custom/path/config.toml
```

If the default config file does not exist next to the installed `oci`
executable, normal commands use built-in defaults. `oci config` creates the
selected file.

## Priority

Runtime priority is:

```text
CLI flags
custom config path passed with --path
installed CLI config at <oci-install-dir>/config.toml
built-in defaults
```

Examples:

```text
oci encode "#E85A9A" --space hex
oci encode "#E85A9A" --space hex --format json
oci encode "#E85A9A" --space hex --precision 4
```

If the installed CLI config sets `output.format = "pretty"`, `--format json`
still returns JSON for that invocation.

## Interactive Config Command

`oci config` opens a terminal wizard when stdin is attached to an interactive
terminal. The wizard lets the user view and edit:

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
- default input color space
- registry source
- registry validation behavior

In non-interactive contexts, `oci config` writes the current or built-in
defaults to the selected TOML path without prompting. This keeps tests and
scripts deterministic.

## TOML Shape

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

## Output Defaults

The default CLI output is `pretty`, not JSON. JSON remains available through:

```text
--format json
```

`plain` is supported for compact output on commands where a concise text form
is useful.

## Registry Source

`registry.source = "bundled"` is the only supported v1-beta runtime source. The
TOML shape reserves `source = "path"` and `registry.path` for future work, but
the current CLI returns `registry_error` rather than loading external registry
data.

`registry.validate_on_start = true` validates the bundled frozen registry before
the command proceeds.

## Errors

Invalid config files return structured CLI errors with code:

```text
config_error
```

Unsupported registry source values return:

```text
registry_error
```
