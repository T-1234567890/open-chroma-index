# Open Chroma Index Documentation

Open Chroma Index (OCI) v1-beta is a Rust reference implementation for
deterministic digital color identity. It converts supported color inputs into
canonical OKLCH/OKLab, maps them to a frozen registered base step, records a
six-decimal offset, and can inspect/export the result.

This `docs/` directory is the documentation root for the repository.

## Documents

- [CLI Commands](cli.md): every `oci` command, arguments, output modes, examples,
  config priority, and error codes.
- [Configuration](configuration.md): installed-location TOML config, defaults,
  path resolution, and the interactive config wizard.
- [Algorithm](algorithm.md): encode, decode, inspect, canonicalization,
  candidate family selection, nearest-step search, offset math, and tie-breaks.
- [ID Format](id-format.md): short/full ID syntax, step-number mapping,
  offsets, and validation rules.
- [Color Math](color-math.md): typed color structs, transfer functions,
  matrices, OKLab/OKLCH formulas, gamut checks, and quantization.
- [Registry And Families](registry.md): frozen registry files, checksums,
  step generation, family classification, and the full 64-family table.
- [Export And Support Matrix](export-support.md): target exports, support
  statuses, round-trip error, and v1-beta unsupported/profile-required areas.
- [Native Rust SDK](rust-api.md): core crate modules, public entry points,
  examples, and important result structures.
- [Local Kernel API](local-api.md): localhost HTTP JSON API endpoints, request
  and response envelopes, errors, and security notes.
- [SVG Swatch Templates](swatch-templates.md): `oci swatch gen`, `oci swatch
  data`, template placeholders, and output rules.
- [Limitations](limitations.md): what v1-beta intentionally does not implement.

## Current Implementation Scope

The repository currently contains:

- Root library package: `open-chroma-index`
- Rust import crate: `oci_core`
- Separate CLI crate: `oci-cli`, binary name `oci`
- Frozen bundled registry: `registry/v1/*.json`
- Registry generator tool: `tools/generate-registry`
- CLI configuration stored beside the installed `oci` executable, not in the
  repository

The runtime library reads frozen JSON with `include_str!` and validates SHA-256
checksums before building the registry. It does not regenerate registered steps
at runtime.

## Quick Start

```text
cargo run -p oci-cli -- encode "#E85A9A" --space hex
cargo run -p oci-cli -- encode "#E85A9A" --space hex --format json
cargo run -p oci-cli -- inspect OCI-1-48RS-327
cargo run -p oci-cli -- registry info
cargo run -p oci-cli -- swatch gen --id OCI-1-22TL-326 --template cli/Color_Cards_OCI_v1.svg --out out/
cargo run -p oci-cli -- serve
cargo run -p oci-cli -- config
```

Default CLI output is a human-readable pretty summary. Use `--format json` for
automation.
