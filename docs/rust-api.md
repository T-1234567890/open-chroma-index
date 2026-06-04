# Rust API

The root package is `open-chroma-index`. Its Rust import crate is `oci_core`.
The CLI crate depends on it by path during development.

## Modules

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

## Main Public Types

Color:

- `EncodedSrgb`, `LinearSrgb`
- `EncodedDisplayP3`, `LinearDisplayP3`
- `EncodedAdobeRgb1998`, `LinearAdobeRgb1998`
- `EncodedRec709`, `LinearRec709`
- `XyzD65`
- `Oklab`
- `Oklch`

OCI ID and registry:

- `FamilyCode`
- `FamilyId`
- `StepId`
- `OklchOffset`
- `OciId`
- `Registry`
- `RegistryStep`
- `Family`

Pipeline:

- `ColorInput`
- `EncodeResult`
- `InspectResult`
- `NearestStep`
- `RegistryIndex`

Export/support:

- `ExportSet`
- `SupportMatrix`
- `SupportEntry`
- `SupportStatus`
- `TargetColorSystem`

Errors:

- `ColorError`
- `OciIdError`
- `RegistryError`
- `OciPipelineError`

## Load Registry

```rust
use oci_core::Registry;

let registry = Registry::load_frozen()?;
```

`load_frozen` validates checksums, parses bundled JSON, and validates registry
invariants.

## Encode HEX

```rust
use oci_core::{encode_from_hex, Registry};

let registry = Registry::load_frozen()?;
let result = encode_from_hex("#E85A9A", &registry)?;

println!("{}", result.short_id);
println!("{}", result.full_id);
println!("{:?}", result.decoded_oklch);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Encode Arbitrary Input

```rust
use oci_core::{encode, ColorInput, EncodedDisplayP3, Registry};

let registry = Registry::load_frozen()?;
let input = ColorInput::DisplayP3Float {
    r: 0.84,
    g: 0.39,
    b: 0.60,
};
let result = encode(input, &registry)?;
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

## Parse And Inspect ID

```rust
use oci_core::{inspect, OciId, Registry};

let registry = Registry::load_frozen()?;
let id = OciId::parse_with_registry("OCI-1-46PK-236", &registry)?;
let inspected = inspect(&id, &registry)?;

println!("{}", inspected.canonical_short_id);
println!("{:?}", inspected.canonical_oklch);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Decode ID

```rust
use oci_core::{decode_oci_id, OciId, Registry};

let registry = Registry::load_frozen()?;
let id = OciId::parse_with_registry(
    "OCI-1-48RS-327@L-0.030857,C-0.010032,H+0.641361",
    &registry,
)?;
let color = decode_oci_id(&id, &registry)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Export

```rust
use oci_core::{export_all, Oklch};

let exports = export_all(Oklch::new(0.669143, 0.185968, 355.308028));
println!("{:?}", exports.hex);
println!("{}", exports.css.oklch);
```

## Support Matrix

```rust
use oci_core::{build_support_matrix, Oklch, TargetColorSystem};

let matrix = build_support_matrix(Oklch::new(0.669143, 0.185968, 355.308028));
let hex_status = matrix.status_for(TargetColorSystem::Hex);
```

## Registry Lookup

```rust
use oci_core::{FamilyId, Registry, StepId};

let registry = Registry::load_frozen()?;
let family = FamilyId::new(46, "PK")?;
let step = StepId::new(2, 12, 6)?;
let record = registry.find_step(family, step).unwrap();
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Important Notes

- The runtime registry is frozen. Do not call the generator in normal library
  use.
- `OciId::parse()` loads the frozen registry internally. Prefer
  `parse_with_registry` if you already have a registry.
- Color conversions use `f64`; exported HEX/RGB8 are quantized.
- Out-of-gamut exports are marked unsupported unless the target can represent
  the color.
