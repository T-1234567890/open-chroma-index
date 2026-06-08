//! Core Open Chroma Index primitives.
//!
//! Part 1 contains typed color values, parsing helpers, gamut checks, and
//! deterministic color-space conversion math. Part 2 adds frozen v1-beta
//! registry loading plus OCI ID parsing and formatting.

pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod color;
pub mod convert;
pub mod error;
pub mod export;
pub mod gamut;
pub mod id;
pub mod index;
pub mod registry;

pub use color::*;
pub use convert::{
    adobe_rgb_1998_to_oklch_to_adobe_rgb_1998, display_p3_to_oklch_to_display_p3,
    rec709_to_oklch_to_rec709, srgb_to_oklch_to_srgb,
};
pub use error::{ColorError, OciIdError, OciPipelineError, RegistryError};
pub use export::{
    ColorExport, CssColorSyntax, ExportSet, FloatRgb, Hsl, JsonColorValue, JsonComponent, Rgb8,
    SupportEntry, SupportMatrix, TargetColorSystem, build_support_matrix, export_all,
};
pub use gamut::{SupportStatus, oklab_distance, shortest_hue_diff};
pub use id::{FamilyCode, FamilyId, OciId, OklchOffset, StepId};
pub use index::{
    ColorInput, EncodeResult, InspectResult, NearestStep, RegistryIndex, candidate_family_indices,
    canonicalize_oci_id, decode_oci_id, encode, encode_from_hex, inspect,
};
pub use registry::{Family, Registry, RegistrySource, RegistryStep};

#[cfg(test)]
mod package_tests {
    #[test]
    fn root_package_excludes_cli_directory() {
        let manifest = include_str!("../Cargo.toml");
        assert!(manifest.contains("exclude"));
        assert!(manifest.contains("\"cli/**\""));
    }
}
