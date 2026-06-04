# Open Chroma Index

Open color IDs for digital design.

Open Chroma Index (OCI) is an open, deterministic, digital-first color identity
standard. It gives colors stable, human-readable IDs that map to canonical
OKLCH/OKLab values, then export deterministically to digital color systems
such as HEX, RGB, HSL, sRGB, Display P3, Adobe RGB 1998, Rec.709, OKLCH, OKLab,
CSS, and design-token formats.

![Rust](https://img.shields.io/badge/Rust-2024-orange)
![License](https://img.shields.io/badge/License-Apache--2.0-blue)
![Status](https://img.shields.io/badge/Status-v1--beta%2Fexperimental-yellow)

## Why OCI?

HEX is useful, but it is not a color identity. It is an 8-bit sRGB encoding,
which means the same intended color may need different representations in wider
gamut spaces, CSS, design tokens, or production workflows.

Proprietary color libraries are important in their own domains, but they are
not portable, open, algorithmic digital identity layers. OCI gives digital
colors stable IDs based on OKLCH/OKLab, then lets tools export those colors to
the representations they need.

OCI is not Pantone data, not a Pantone clone, and not a shortcut around licensed
production color systems.

OCI exists one layer earlier: it gives digital colors stable, open, algorithmic
identities before they are exported to design tools, code, or production
workflows.

## What OCI Is

- A canonical OKLCH/OKLab identity model.
- A frozen v1-beta registry of named families and registered base steps.
- A short and full OCI ID format.
- An optional OKLCH offset for precision beyond the registered base step.
- Deterministic exports to supported digital color systems.
- A support matrix that reports whether each target representation is supported,
  lossy, unsupported, or profile-required.

## What OCI Is Not

- Not Pantone data.
- Not RAL data.
- Not a physical print guarantee.
- Not universal CMYK.
- Not a replacement for production proofing.
- Not only a palette generator.

Pantone and RAL may be user-supplied references or future licensed integrations,
but official libraries are not included in this repository. CMYK numeric
conversion requires an ICC/profile workflow and is currently reported as
`profile_required`.

## Example

Example produced by the current v1-beta registry. These exact numeric values are
illustrative and may change before a future stable standard.

Input:

```text
#E85A9A
```

Current pretty CLI output:

```text
OCI Encode
input: #E85A9A (hex)

OCI standard color code: OCI-1-48RS-327
OCI precision color code: OCI-1-48RS-327@L-0.030857,C-0.010032,H+0.641361
oklch: L=0.669143 C=0.185968 H=355.308028

exports:
  hex: #E85A9A (lossy, round-trip error 0.000000348709)
  oklch: L=0.669143 C=0.185968 H=355.308028
  display-p3: r=0.844459 g=0.389513 b=0.597250 (supported, round-trip error 0.000000024397)
  css oklch: oklch(66.914300% 0.185968 355.308028deg)
  css srgb: rgb(232 90 154)
  css display-p3: color(display-p3 0.844459 0.389513 0.597250)

support: 12 targets evaluated
warnings: none
```

The standard color code is the registered base step. The precision color code is
the same base step plus the OKLCH offset needed to reconstruct the canonical
input more closely.

## ID Format

Example short standard and precision forms:

```text
OCI-1-48RS-327
OCI-1-48RS-327@L-0.030857,C-0.010032,H+0.641361
```

Structural short forms:

```text
OCI-{majorVersion}-{familyIndex}{familyCode}-{stepNumber}
OCI-{majorVersion}-{familyIndex}{familyCode}-{stepNumber}@L{lightnessOffset},C{chromaOffset},H{hueOffset}
```

Structural full forms:

```text
OCI-{majorVersion}-{familyIndex}{familyCode}-A{anchor}-L{lightnessLevel}-C{chromaLevel}
OCI-{majorVersion}-{familyIndex}{familyCode}-A{anchor}-L{lightnessLevel}-C{chromaLevel}@L{lightnessOffset},C{chromaOffset},H{hueOffset}
```

Short and full forms resolve to the same registered base step. The offset is
optional. A standard color code is the base registered step. A precision color
code is the base registered step plus offset.

Exact numeric examples in this README are illustrative and may change before
v1 stable.

## Registry

Runtime registry data lives in `registry/v1/`:

- `registry/v1/families.json`
- `registry/v1/steps.json`
- `registry/v1/test-vectors.json`
- `registry/v1/schema.json`
- `registry/v1/checksums.json`
- `registry/v1/metadata.json`

The v1-beta registry currently contains:

- 64 families
- 360 steps per family
- 23,040 registered base steps total

The registry is frozen data at runtime. The generator may produce registry
files, but the runtime kernel reads frozen registry data and does not
dynamically generate registered steps.

## Supported Color Systems

Canonical:

- OKLCH
- OKLab

Digital:

- HEX
- RGB
- HSL
- sRGB
- Display P3
- Adobe RGB 1998
- Rec.709
- CSS output
- JSON tokens

Production-adjacent:

- Lab references are production-adjacent, but the current kernel is canonical
  OKLab/OKLCH rather than a full CIELAB workflow.
- CMYK is `profile_required` for now.
- Pantone and RAL are user-supplied references or future licensed integrations
  only.

## CLI

The CLI is a separate crate under `cli/`. It is a command-line client for the
root OCI kernel/library crate.

See [cli/README.md](cli/README.md).

Basic commands:

```text
oci encode "#E85A9A" --space hex
oci inspect OCI-1-48RS-327
oci export OCI-1-48RS-327 --to hex,oklch,css
oci config
```

## Rust Library Usage

The root crate is the OCI kernel/library crate. The Rust API is experimental
while OCI is in v1-beta.

```rust
use oci_core::{encode_from_hex, Registry};

let registry = Registry::load_frozen()?;
let encoded = encode_from_hex("#E85A9A", &registry)?;

println!("{}", encoded.short_id);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Repository Layout

```text
open-chroma-index/
  Cargo.toml              # root open-chroma-index kernel/library package
  README.md               # main project README
  src/                    # color math, registry, ID, export, and index kernel
  cli/                    # separate oci-cli crate, binary name oci
  registry/               # frozen registry data
  spec/                   # draft specification notes
  docs/                   # implementation documentation
  tools/generate-registry # registry generator
```

## Installation

OCI is currently v1-beta / experimental. Installation commands may change before
v1 stable.

Use the Rust library if you want to integrate OCI into another Rust project.
Use the CLI if you want the `oci` command in your terminal.

### Rust library / kernel

The root crate is the OCI kernel/library crate. It is meant for Rust apps,
tools, plugins, and other libraries that need OCI color conversion and identity
logic.

Published dependency:

```toml
[dependencies]
open-chroma-index = "0.1"
```

Local development dependency:

```toml
[dependencies]
open-chroma-index = { path = "." }
```

This installs or uses the kernel library only. It does not install the `oci`
command-line binary.

### CLI via Cargo

The CLI is a separate crate under `cli/`. It depends on the kernel crate and is
released separately from the kernel/library crate.

From crates.io:

```bash
cargo install oci-cli
```

The installed binary is:

```bash
oci
```

From local source:

```bash
cargo install --path cli
```

During development:

```bash
cargo run -p oci-cli -- encode "#E85A9A" --space hex
```

Cargo install is best for Rust users who already have a Rust toolchain.

### CLI via install script

The install script is for users who want the `oci` CLI binary without using
Cargo directly. It installs the CLI binary, not the kernel crate. The script
downloads release artifacts from GitHub Releases.

Install the latest `cli-v*` release:

```bash
curl -fsSL https://raw.githubusercontent.com/T-1234567890/open-chroma-index/main/install.sh | bash
```

Install a specific CLI release tag:

```bash
curl -fsSL https://raw.githubusercontent.com/T-1234567890/open-chroma-index/main/install.sh | bash -s -- --version cli-v0.1.0
```

Install to a custom directory:

```bash
curl -fsSL https://raw.githubusercontent.com/T-1234567890/open-chroma-index/main/install.sh | bash -s -- --dir ~/.local/bin
```

System install:

```bash
curl -fsSL https://raw.githubusercontent.com/T-1234567890/open-chroma-index/main/install.sh | bash -s -- --system
```

CLI release tags use `cli-v*`. Kernel/library release tags use `core-v*`. Do
not confuse the two: `cli-v*` releases contain CLI binaries, while `core-v*`
releases contain the kernel/library crate and registry/spec assets.

### Manual release download

You can also download CLI binaries manually from GitHub Releases.

Expected CLI artifacts:

```text
oci-x86_64-unknown-linux-gnu.tar.gz
oci-aarch64-apple-darwin.tar.gz
oci-x86_64-apple-darwin.tar.gz
oci-x86_64-pc-windows-msvc.zip
```

Checksum files:

```text
oci-x86_64-unknown-linux-gnu.sha256
oci-aarch64-apple-darwin.sha256
oci-x86_64-apple-darwin.sha256
oci-x86_64-pc-windows-msvc.sha256
```

Platform mapping:

| Platform | Artifact |
| --- | --- |
| Linux x86_64 | `oci-x86_64-unknown-linux-gnu.tar.gz` |
| macOS Apple Silicon | `oci-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `oci-x86_64-apple-darwin.tar.gz` |
| Windows x86_64 | `oci-x86_64-pc-windows-msvc.zip` |

### Verify installation

```bash
oci --help
oci registry info
oci encode "#E85A9A" --space hex
```

Default CLI output is human-readable pretty output. JSON is available for
automation:

```bash
oci encode "#E85A9A" --space hex --format json
```

## Development

Run checks during development:

```bash
cargo test
```

## Publishing Notes

- The root crate is the core kernel crate.
- The root crate package excludes `cli/**`.
- The CLI crate is published separately.
- The CLI crate depends on the core crate by path during development.
- Do not bundle the CLI crate inside the core crate package.

## Licensing

License: Apache-2.0.

## Status/Roadmap

Current:

```text
v1-beta
```

Not yet:

- ICC CMYK numeric conversion
- Pantone/RAL official integration
- GUI
- web playground
- plugins

## Contributing

- Do not casually change frozen registry data.
- Family and step table changes require versioning.
- Tests and test vectors are required for behavior changes.
- Keep documentation clear about what is implemented, what is experimental, and
  what requires external licensed data or production profiles.

## More Documentation

Full implementation documentation lives in [`docs/`](docs/README.md).
