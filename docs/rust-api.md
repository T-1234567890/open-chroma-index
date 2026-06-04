# Native Rust SDK

This page explains how to use Open Chroma Index as a Rust dependency.

Use the native Rust SDK if you want to integrate OCI into another Rust project.
Use the CLI if you want the `oci` command in your terminal.

## Package Name And Import Name

The published package name is:

```toml
open-chroma-index
```

The Rust import crate name is:

```rust
oci_core
```

This means your `Cargo.toml` uses the package name, while Rust source files use
the import crate name:

```toml
[dependencies]
open-chroma-index = "<published-version>"
```

```rust
use oci_core::{Registry, encode_from_hex};
```

For local development against this repository:

```toml
[dependencies]
open-chroma-index = { path = "../open-chroma-index" }
```

If your project is inside this repository, adjust the `path` value to the
relative location of the repository root.

Installing the library dependency does not install the `oci` command-line
binary. The CLI is the separate `oci-cli` crate under `cli/`.

## Status

The native Rust SDK is experimental. The current API is usable for apps, tools,
plugins, and libraries, but names and result structures may still change before
a future stable OCI release.

The runtime registry is frozen. Normal library code should load the bundled
registry and should not call the registry generator.

## Main Modules

```rust
pub mod color;
pub mod convert;
pub mod error;
pub mod export;
pub mod gamut;
pub mod id;
pub mod index;
pub mod registry;
```

Most applications can use the public re-exports from `oci_core` directly rather
than importing module paths.

## Main Public Types

Color types:

- `EncodedSrgb`, `LinearSrgb`
- `EncodedDisplayP3`, `LinearDisplayP3`
- `EncodedAdobeRgb1998`, `LinearAdobeRgb1998`
- `EncodedRec709`, `LinearRec709`
- `XyzD65`
- `Oklab`
- `Oklch`

OCI ID and registry types:

- `FamilyCode`
- `FamilyId`
- `StepId`
- `OklchOffset`
- `OciId`
- `Registry`
- `RegistryStep`
- `Family`

Pipeline types:

- `ColorInput`
- `EncodeResult`
- `InspectResult`
- `NearestStep`
- `RegistryIndex`

Export and support types:

- `ExportSet`
- `SupportMatrix`
- `SupportEntry`
- `SupportStatus`
- `TargetColorSystem`

Error types:

- `ColorError`
- `OciIdError`
- `RegistryError`
- `OciPipelineError`

## Load The Frozen Registry

Most API calls need a `Registry`.

```rust
use oci_core::Registry;

let registry = Registry::load_frozen()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

`Registry::load_frozen()` reads the bundled `registry/v1/*.json` data compiled
into the crate, validates SHA-256 checksums, and validates registry invariants.
It does not dynamically regenerate registered steps.

If you call several OCI functions, load the registry once and pass `&registry`
around.

## Encode From HEX

```rust
use oci_core::{Registry, encode_from_hex};

let registry = Registry::load_frozen()?;
let result = encode_from_hex("#E85A9A", &registry)?;

println!("short: {}", result.short_id);
println!("full: {}", result.full_id);
println!("canonical OKLCH: {:?}", result.decoded_oklch);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`encode_from_hex` is a convenience wrapper for HEX input. HEX is interpreted as
8-bit encoded sRGB.

## Encode Other Supported Inputs

Use `encode` with `ColorInput` when the source is not HEX.

```rust
use oci_core::{ColorInput, Registry, encode};

let registry = Registry::load_frozen()?;
let input = ColorInput::DisplayP3Float {
    r: 0.844459,
    g: 0.389513,
    b: 0.597250,
};

let result = encode(input, &registry)?;

println!("{}", result.short_id);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Supported `ColorInput` variants:

- `Hex(String)`
- `Srgb(EncodedSrgb)`
- `SrgbRgb { r: u8, g: u8, b: u8 }`
- `HslSrgb { h: f64, s: f64, l: f64 }`
- `DisplayP3Float { r: f64, g: f64, b: f64 }`
- `AdobeRgb1998Float { r: f64, g: f64, b: f64 }`
- `Rec709Float { r: f64, g: f64, b: f64 }`
- `Oklch(Oklch)`
- `Oklab(Oklab)`
- `OciId(OciId)`
- `OciIdString(String)`

## Encode Result Fields

`EncodeResult` includes:

- `input_oklch`: the input converted to canonical OKLCH.
- `oci_id`: parsed structured OCI ID with optional offset.
- `short_id`: short precision ID string.
- `full_id`: full precision ID string.
- `decoded_oklch`: OKLCH decoded back from the OCI ID.
- `encoding_error`: OKLab distance between input and decoded color.
- `nearest_step`: selected registered base step and distance.
- `exports`: exported target color values.
- `support_matrix`: support status per target color system.

For a standard color code without offset, clone `result.oci_id`, set
`offset = None`, and format with `to_short_string()` or `to_full_string()`.

## Parse And Inspect An OCI ID

Use `parse_with_registry` when you already have a registry.

```rust
use oci_core::{OciId, Registry, inspect};

let registry = Registry::load_frozen()?;
let id = OciId::parse_with_registry("OCI-1-48RS-327", &registry)?;
let inspected = inspect(&id, &registry)?;

println!("short: {}", inspected.short_id);
println!("full: {}", inspected.full_id);
println!("canonical short: {}", inspected.canonical_short_id);
println!("canonical OKLCH: {:?}", inspected.canonical_oklch);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`InspectResult` includes the input ID forms, canonicalized ID forms, canonical
OKLCH, exports, and support matrix.

## Decode An OCI ID To OKLCH

```rust
use oci_core::{OciId, Registry, decode_oci_id};

let registry = Registry::load_frozen()?;
let id = OciId::parse_with_registry(
    "OCI-1-48RS-327@L-0.030857,C-0.010032,H+0.641361",
    &registry,
)?;

let color = decode_oci_id(&id, &registry)?;
println!("L={} C={} H={}", color.l, color.c, color.h);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Decoding looks up the frozen base step, applies any offset, normalizes the
result, and returns canonical OKLCH.

## Export A Canonical Color

```rust
use oci_core::{Oklch, export_all};

let color = Oklch::new(0.669143, 0.185968, 355.308028);
let exports = export_all(color);

println!("{:?}", exports.hex);
println!("{}", exports.css.oklch);
```

`ExportSet` includes sRGB float, HEX, RGB8, HSL, Display P3, Adobe RGB 1998,
Rec.709, OKLCH, OKLab, CSS syntax, JSON-friendly values, and a CMYK placeholder
status.

CMYK numeric conversion is not implemented. It is reported as
`profile_required`.

## Build A Support Matrix

```rust
use oci_core::{Oklch, TargetColorSystem, build_support_matrix};

let color = Oklch::new(0.669143, 0.185968, 355.308028);
let matrix = build_support_matrix(color);

let hex_status = matrix.status_for(TargetColorSystem::Hex);
println!("{hex_status:?}");
```

The support matrix reports whether each target representation is supported,
lossy, unsupported, profile-required, or another explicit support status.

HEX and RGB8 are lossy because they quantize through 8-bit sRGB. Out-of-gamut
targets are not silently clamped and reported as supported.

## Registry Lookup

```rust
use oci_core::{FamilyId, Registry, StepId};

let registry = Registry::load_frozen()?;
let family = FamilyId::new(48, "RS")?;
let step = StepId::new(3, 9, 7)?;
let record = registry.find_step(family, step).unwrap();

println!("{}", record.short_id);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Registry lookup is useful when tooling needs direct access to frozen family and
step records.

## Error Handling

OCI APIs return typed errors:

- `ColorError`: color parsing or channel validation errors.
- `OciIdError`: OCI ID parsing, family, step, or offset errors.
- `RegistryError`: frozen registry parsing, checksum, or invariant errors.
- `OciPipelineError`: encode/decode/inspect pipeline errors.

Applications can bubble these errors with `?`, map them into application error
types, or display their messages to users.

## Practical Integration Notes

- Load `Registry::load_frozen()` once and reuse it.
- Use `encode_from_hex` for simple HEX workflows.
- Use `encode(ColorInput, &registry)` for typed inputs.
- Use `inspect` when users provide an OCI ID and you need exports/support.
- Use `decode_oci_id` when you only need canonical OKLCH.
- Use `export_all` when you already have OKLCH and need target representations.
- Keep CLI installation separate from Rust library dependency setup.
