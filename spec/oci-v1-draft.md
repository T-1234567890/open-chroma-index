# OCI v1-beta Draft

Open Chroma Index v1-beta defines a deterministic color identity system based
on canonical OKLCH/OKLab values, frozen family labels, frozen registered base
steps, and precision offsets.

v1-beta is not a frozen production standard. It is a reference implementation
for developer testing and interoperability experiments.

Included:
- Typed color math for sRGB, Display P3, Adobe RGB 1998, Rec.709, XYZ D65,
  OKLab, and OKLCH.
- Frozen registry data with 64 families and 23,040 registered base steps.
- OCI short and full IDs with optional OKLCH offsets.
- A CLI named `oci` and a Rust core crate.
- TOML CLI configuration with pretty output by default and JSON available
  through `--format json`.

Not included:
- Pantone and RAL official libraries.
- CMYK numeric conversion. CMYK requires ICC profile support.
- GUI, web app, cloud service, plugins, or production print proofing.

Physical print proofing is still required for production.

Related spec notes:
- `id-format.md` defines short/full OCI ID syntax and offsets.
- `color-spaces.md` defines canonical and representation color spaces.
- `registry-data.md` defines frozen family and step data behavior.
- `support-matrix.md` defines export support statuses.
- `cli-configuration.md` defines CLI TOML configuration behavior.
