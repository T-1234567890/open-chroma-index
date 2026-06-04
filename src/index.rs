use crate::color::{
    EncodedAdobeRgb1998, EncodedDisplayP3, EncodedRec709, EncodedSrgb, Oklab, Oklch,
};
use crate::error::OciPipelineError;
use crate::export::{ExportSet, SupportMatrix, build_support_matrix, export_all, normalize_oklch};
use crate::gamut::{oklab_distance, shortest_hue_diff};
use crate::id::{FamilyId, OciId, OklchOffset, StepId};
use crate::registry::{Registry, RegistryStep};
use std::collections::HashMap;

const OFFSET_SCALE: f64 = 1_000_000.0;
const TIE_EPSILON: f64 = 1.0e-12;
const NEUTRAL_CHROMA_LIMIT: f64 = 0.035;
const MUTED_CHROMA_LIMIT: f64 = 0.17;

#[derive(Debug, Clone, PartialEq)]
pub enum ColorInput {
    Hex(String),
    Srgb(EncodedSrgb),
    SrgbRgb { r: u8, g: u8, b: u8 },
    HslSrgb { h: f64, s: f64, l: f64 },
    DisplayP3Float { r: f64, g: f64, b: f64 },
    AdobeRgb1998Float { r: f64, g: f64, b: f64 },
    Rec709Float { r: f64, g: f64, b: f64 },
    Oklch(Oklch),
    Oklab(Oklab),
    OciId(OciId),
    OciIdString(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NearestStep {
    pub family_id: FamilyId,
    pub step: StepId,
    pub full_id: String,
    pub short_id: String,
    pub base_oklch: Oklch,
    pub distance: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EncodeResult {
    pub input_oklch: Oklch,
    pub oci_id: OciId,
    pub short_id: String,
    pub full_id: String,
    pub decoded_oklch: Oklch,
    pub encoding_error: f64,
    pub nearest_step: NearestStep,
    pub exports: ExportSet,
    pub support_matrix: SupportMatrix,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InspectResult {
    pub oci_id: OciId,
    pub short_id: String,
    pub full_id: String,
    pub canonical_oklch: Oklch,
    pub canonical_id: OciId,
    pub canonical_short_id: String,
    pub canonical_full_id: String,
    pub exports: ExportSet,
    pub support_matrix: SupportMatrix,
}

#[derive(Debug, Clone)]
pub struct RegistryIndex {
    family_by_id: HashMap<FamilyId, usize>,
    family_steps_by_index: HashMap<u8, Vec<usize>>,
    step_by_full_id: HashMap<String, usize>,
    step_by_short_id: HashMap<String, usize>,
}

impl RegistryIndex {
    #[must_use]
    pub fn from_registry(registry: &Registry) -> Self {
        let family_by_id = registry
            .families()
            .iter()
            .enumerate()
            .map(|(position, family)| (family.id, position))
            .collect();

        let mut family_steps_by_index: HashMap<u8, Vec<usize>> = HashMap::new();
        let mut step_by_full_id = HashMap::with_capacity(registry.steps().len());
        let mut step_by_short_id = HashMap::with_capacity(registry.steps().len());
        for (position, step) in registry.steps().iter().enumerate() {
            family_steps_by_index
                .entry(step.family_id.index)
                .or_default()
                .push(position);
            step_by_full_id.insert(step.id.clone(), position);
            step_by_short_id.insert(step.short_id.clone(), position);
        }

        Self {
            family_by_id,
            family_steps_by_index,
            step_by_full_id,
            step_by_short_id,
        }
    }

    #[must_use]
    pub fn family_position(&self, family_id: FamilyId) -> Option<usize> {
        self.family_by_id.get(&family_id).copied()
    }

    #[must_use]
    pub fn step_by_oci_id<'a>(
        &self,
        registry: &'a Registry,
        id: &OciId,
    ) -> Option<&'a RegistryStep> {
        let key = format!("OCI-{}-{}-{}", id.version, id.family, id.step);
        self.step_by_full_id
            .get(&key)
            .and_then(|position| registry.steps().get(*position))
    }

    #[must_use]
    pub fn step_by_short_id<'a>(
        &self,
        registry: &'a Registry,
        short_id: &str,
    ) -> Option<&'a RegistryStep> {
        self.step_by_short_id
            .get(short_id)
            .and_then(|position| registry.steps().get(*position))
    }

    #[must_use]
    pub fn steps_for_family<'a>(
        &self,
        registry: &'a Registry,
        family_index: u8,
    ) -> Vec<&'a RegistryStep> {
        self.family_steps_by_index
            .get(&family_index)
            .into_iter()
            .flat_map(|positions| positions.iter())
            .filter_map(|position| registry.steps().get(*position))
            .collect()
    }
}

impl ColorInput {
    pub fn to_oklch(&self, registry: &Registry) -> Result<Oklch, OciPipelineError> {
        match self {
            Self::Hex(hex) => Ok(EncodedSrgb::from_hex(hex)?.to_oklch()),
            Self::Srgb(color) => {
                validate_finite_rgb("sRGB", color.r, color.g, color.b)?;
                Ok(color.to_oklch())
            }
            Self::SrgbRgb { r, g, b } => Ok(EncodedSrgb::from_rgb_u8(*r, *g, *b)?.to_oklch()),
            Self::HslSrgb { h, s, l } => Ok(EncodedSrgb::from_hsl(*h, *s, *l)?.to_oklch()),
            Self::DisplayP3Float { r, g, b } => {
                validate_finite_rgb("Display P3", *r, *g, *b)?;
                Ok(EncodedDisplayP3::new(*r, *g, *b).to_oklch())
            }
            Self::AdobeRgb1998Float { r, g, b } => {
                validate_finite_rgb("Adobe RGB 1998", *r, *g, *b)?;
                Ok(EncodedAdobeRgb1998::new(*r, *g, *b).to_oklch())
            }
            Self::Rec709Float { r, g, b } => {
                validate_finite_rgb("Rec.709", *r, *g, *b)?;
                Ok(EncodedRec709::new(*r, *g, *b).to_oklch())
            }
            Self::Oklch(color) => {
                validate_finite_oklch(*color)?;
                Ok(normalize_oklch(*color))
            }
            Self::Oklab(color) => {
                validate_finite_oklab(*color)?;
                Ok(normalize_oklch(color.to_oklch()))
            }
            Self::OciId(id) => decode_oci_id(id, registry),
            Self::OciIdString(id) => {
                let parsed = OciId::parse_with_registry(id, registry)?;
                decode_oci_id(&parsed, registry)
            }
        }
    }
}

pub fn encode(input: ColorInput, registry: &Registry) -> Result<EncodeResult, OciPipelineError> {
    let input_oklch = input.to_oklch(registry)?;
    encode_oklch(input_oklch, registry)
}

pub fn encode_from_hex(hex: &str, registry: &Registry) -> Result<EncodeResult, OciPipelineError> {
    encode(ColorInput::Hex(hex.to_string()), registry)
}

pub fn inspect(id: &OciId, registry: &Registry) -> Result<InspectResult, OciPipelineError> {
    let canonical_oklch = decode_oci_id(id, registry)?;
    let canonical_id = canonicalize_oci_id(id, registry)?;
    let exports = export_all(canonical_oklch);
    let support_matrix = build_support_matrix(canonical_oklch);

    Ok(InspectResult {
        oci_id: id.clone(),
        short_id: id.to_short_string(),
        full_id: id.to_full_string(),
        canonical_oklch,
        canonical_short_id: canonical_id.to_short_string(),
        canonical_full_id: canonical_id.to_full_string(),
        canonical_id,
        exports,
        support_matrix,
    })
}

pub fn canonicalize_oci_id(id: &OciId, registry: &Registry) -> Result<OciId, OciPipelineError> {
    Ok(encode_oklch(decode_oci_id(id, registry)?, registry)?.oci_id)
}

pub fn decode_oci_id(id: &OciId, registry: &Registry) -> Result<Oklch, OciPipelineError> {
    let Some(step) = registry.find_step(id.family, id.step) else {
        return Err(OciPipelineError::MissingBaseStep {
            id: id.to_full_string(),
        });
    };

    let mut decoded = step_to_oklch(step);
    if let Some(offset) = id.offset {
        decoded.l += offset.lightness;
        decoded.c += offset.chroma;
        decoded.h += offset.hue;
    }

    Ok(normalize_oklch(decoded))
}

pub fn nearest_registered_step(
    color: Oklch,
    registry: &Registry,
) -> Result<NearestStep, OciPipelineError> {
    let index = RegistryIndex::from_registry(registry);
    nearest_registered_step_with_index(color, registry, &index)
}

pub fn nearest_registered_step_with_index(
    color: Oklch,
    registry: &Registry,
    index: &RegistryIndex,
) -> Result<NearestStep, OciPipelineError> {
    let canonical = normalize_oklch(color);
    let input_lab = canonical.to_oklab();
    let candidate_indices = candidate_family_indices(canonical);
    let mut best: Option<NearestStep> = None;

    for family_index in candidate_indices {
        for step in index.steps_for_family(registry, family_index) {
            let base_oklch = step_to_oklch(step);
            let distance = oklab_distance(input_lab, base_oklch.to_oklab());
            let candidate = NearestStep {
                family_id: step.family_id,
                step: step.step,
                full_id: step.id.clone(),
                short_id: step.short_id.clone(),
                base_oklch,
                distance,
            };

            if match best.as_ref() {
                Some(current) => nearest_candidate_is_better(&candidate, current),
                None => true,
            } {
                best = Some(candidate);
            }
        }
    }

    best.ok_or(OciPipelineError::NoNearestStep)
}

#[must_use]
pub fn candidate_family_indices(color: Oklch) -> Vec<u8> {
    let canonical = normalize_oklch(color);

    if canonical.c <= NEUTRAL_CHROMA_LIMIT {
        return neutral_candidate_family_indices(canonical);
    }

    if let Some(indices) = muted_candidate_family_indices(canonical) {
        return indices;
    }

    chromatic_candidate_family_indices(canonical.h)
}

#[must_use]
pub fn current_chromatic_family_index(hue: f64) -> u8 {
    let normalized = if hue.is_finite() {
        hue.rem_euclid(360.0)
    } else {
        0.0
    };

    for family_index in 0u8..=49 {
        if let Some((start, end)) = chromatic_family_range(family_index)
            && hue_is_in_range(normalized, start, end)
        {
            return family_index;
        }
    }

    0
}

fn chromatic_candidate_family_indices(hue: f64) -> Vec<u8> {
    let current = current_chromatic_family_index(hue);
    let previous = if current == 0 { 49 } else { current - 1 };
    let next = if current == 49 { 0 } else { current + 1 };

    vec![current, previous, next]
}

fn neutral_candidate_family_indices(color: Oklch) -> Vec<u8> {
    if color.l <= 0.18 {
        return vec![63];
    }
    if color.l >= 0.86 {
        return vec![62];
    }
    if hue_is_in_range(color.h, 190.0, 270.0) && color.c > 0.012 {
        return vec![58, 59, 60, 61];
    }
    if color.c <= 0.006 {
        return vec![61, 60, 59];
    }
    if color.c <= 0.012 {
        return vec![60, 61, 59];
    }
    vec![60, 59, 58, 61]
}

fn muted_candidate_family_indices(color: Oklch) -> Option<Vec<u8>> {
    let hue = color.h;
    let chroma = color.c;
    let lightness = color.l;

    if chroma <= MUTED_CHROMA_LIMIT
        && lightness <= 0.52
        && (hue_is_in_range(hue, 335.0, 360.0) || hue_is_in_range(hue, 0.0, 15.0))
    {
        return Some(vec![50, 49, 48]);
    }

    if chroma <= MUTED_CHROMA_LIMIT && lightness <= 0.55 && hue_is_in_range(hue, 15.0, 55.0) {
        return Some(vec![52, 53, 54]);
    }

    if chroma <= 0.14 && lightness > 0.50 && hue_is_in_range(hue, 35.0, 90.0) {
        return Some(vec![54, 55, 52]);
    }

    if chroma <= MUTED_CHROMA_LIMIT && hue_is_in_range(hue, 65.0, 135.0) {
        return Some(vec![56, 57]);
    }

    if chroma <= 0.06 && (0.22..=0.68).contains(&lightness) && hue_is_in_range(hue, 190.0, 270.0) {
        return Some(vec![58, 59]);
    }

    None
}

fn chromatic_family_range(index: u8) -> Option<(f64, f64)> {
    match index {
        0 => Some((0.0, 8.0)),
        1..=44 => {
            let start = 8.0 + f64::from(index - 1) * (312.0 / 44.0);
            let end = 8.0 + f64::from(index) * (312.0 / 44.0);
            Some((start, end))
        }
        45 => Some((320.0, 330.0)),
        46 => Some((330.0, 340.0)),
        47 => Some((340.0, 348.0)),
        48 => Some((348.0, 356.0)),
        49 => Some((356.0, 360.0)),
        _ => None,
    }
}

fn hue_is_in_range(hue: f64, start: f64, end: f64) -> bool {
    let hue = hue.rem_euclid(360.0);
    let start = start.rem_euclid(360.0);
    let end = end.rem_euclid(360.0);

    if start <= end {
        hue >= start && hue < end
    } else {
        hue >= start || hue < end
    }
}

fn encode_oklch(color: Oklch, registry: &Registry) -> Result<EncodeResult, OciPipelineError> {
    validate_finite_oklch(color)?;
    let input_oklch = normalize_oklch(color);
    let nearest_step = nearest_registered_step(input_oklch, registry)?;
    let offset = rounded_offset(input_oklch, nearest_step.base_oklch);
    let offset = if offset_is_zero(offset) {
        None
    } else {
        Some(offset)
    };
    let oci_id = OciId::new(nearest_step.family_id, nearest_step.step, offset);
    let decoded_oklch = decode_oci_id(&oci_id, registry)?;
    let encoding_error = oklab_distance(input_oklch.to_oklab(), decoded_oklch.to_oklab());
    let exports = export_all(decoded_oklch);
    let support_matrix = build_support_matrix(decoded_oklch);

    Ok(EncodeResult {
        input_oklch,
        short_id: oci_id.to_short_string(),
        full_id: oci_id.to_full_string(),
        oci_id,
        decoded_oklch,
        encoding_error,
        nearest_step,
        exports,
        support_matrix,
    })
}

fn nearest_candidate_is_better(candidate: &NearestStep, current: &NearestStep) -> bool {
    if candidate.distance + TIE_EPSILON < current.distance {
        return true;
    }
    if (candidate.distance - current.distance).abs() > TIE_EPSILON {
        return false;
    }

    (
        candidate.family_id.index,
        candidate.step.anchor,
        candidate.step.lightness,
        candidate.step.chroma,
    ) < (
        current.family_id.index,
        current.step.anchor,
        current.step.lightness,
        current.step.chroma,
    )
}

fn rounded_offset(input: Oklch, base: Oklch) -> OklchOffset {
    OklchOffset::new(
        round_offset(input.l - base.l),
        round_offset(input.c - base.c),
        round_offset(shortest_hue_diff(input.h, base.h)),
    )
}

fn round_offset(value: f64) -> f64 {
    let rounded = (value * OFFSET_SCALE).round() / OFFSET_SCALE;
    if rounded.abs() < 0.5 / OFFSET_SCALE {
        0.0
    } else {
        rounded
    }
}

fn offset_is_zero(offset: OklchOffset) -> bool {
    offset.lightness == 0.0 && offset.chroma == 0.0 && offset.hue == 0.0
}

fn step_to_oklch(step: &RegistryStep) -> Oklch {
    Oklch::new(step.lightness, step.chroma, step.hue)
}

fn validate_finite_rgb(
    model: &'static str,
    r: f64,
    g: f64,
    b: f64,
) -> Result<(), OciPipelineError> {
    validate_component(model, r)?;
    validate_component(model, g)?;
    validate_component(model, b)
}

fn validate_finite_oklab(color: Oklab) -> Result<(), OciPipelineError> {
    validate_component("OKLab L", color.l)?;
    validate_component("OKLab a", color.a)?;
    validate_component("OKLab b", color.b)
}

fn validate_finite_oklch(color: Oklch) -> Result<(), OciPipelineError> {
    validate_component("OKLCH L", color.l)?;
    validate_component("OKLCH C", color.c)?;
    validate_component("OKLCH H", color.h)
}

fn validate_component(component: &'static str, value: f64) -> Result<(), OciPipelineError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(OciPipelineError::InvalidColorComponent { component, value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::TargetColorSystem;
    use crate::gamut::SupportStatus;

    const EPS: f64 = 1.0e-6;

    fn assert_close(actual: f64, expected: f64, epsilon: f64) {
        assert!(
            (actual - expected).abs() <= epsilon,
            "expected {actual} to be within {epsilon} of {expected}"
        );
    }

    #[test]
    fn indexes_frozen_registry_steps() {
        let registry = Registry::load_frozen().unwrap();
        let index = RegistryIndex::from_registry(&registry);
        let id = OciId::parse_with_registry("OCI-1-46PK-236", &registry).unwrap();

        assert_eq!(index.family_position(id.family), Some(46));
        assert_eq!(
            index.step_by_oci_id(&registry, &id).unwrap().id,
            "OCI-1-46PK-A2-L12-C06"
        );
        assert_eq!(
            index
                .step_by_short_id(&registry, "OCI-1-46PK-236")
                .unwrap()
                .id,
            "OCI-1-46PK-A2-L12-C06"
        );
    }

    #[test]
    fn encode_from_hex_returns_valid_oci_id() {
        let registry = Registry::load_frozen().unwrap();
        let result = encode_from_hex("#E85A9A", &registry).unwrap();

        assert!(result.short_id.starts_with("OCI-1-"));
        assert!(result.full_id.starts_with("OCI-1-"));
        assert!(result.full_id.contains("-A"));
        assert!(OciId::parse_with_registry(&result.short_id, &registry).is_ok());
        assert!(OciId::parse_with_registry(&result.full_id, &registry).is_ok());
    }

    #[test]
    fn high_chroma_pink_does_not_encode_to_neutral_family() {
        let registry = Registry::load_frozen().unwrap();
        let result = encode_from_hex("#E85A9A", &registry).unwrap();
        let code = result.oci_id.family.code.as_str();

        assert_ne!(code, "BK");
        assert!(
            ["PK", "RP", "RS", "WR", "RD"].contains(&code),
            "unexpected high-chroma pink family: {}",
            result.short_id
        );
        assert!(
            ![58, 59, 60, 61, 62, 63].contains(&result.oci_id.family.index),
            "high-chroma pink must not map to neutral family: {}",
            result.short_id
        );
    }

    #[test]
    fn neutral_lightness_conditions_select_black_white_and_gray() {
        let registry = Registry::load_frozen().unwrap();

        let black = encode_from_hex("#000000", &registry).unwrap();
        assert_eq!(black.oci_id.family.code.as_str(), "BK");

        let white = encode_from_hex("#FFFFFF", &registry).unwrap();
        assert_eq!(white.oci_id.family.code.as_str(), "WH");

        let gray = encode_from_hex("#777777", &registry).unwrap();
        assert!(
            ["GY", "NG", "SG"].contains(&gray.oci_id.family.code.as_str()),
            "unexpected gray family: {}",
            gray.short_id
        );
    }

    #[test]
    fn brown_like_orange_uses_earth_muted_family() {
        let registry = Registry::load_frozen().unwrap();
        let result = encode_from_hex("#8B4513", &registry).unwrap();

        assert!(
            ["BR", "CP", "TN", "BG"].contains(&result.oci_id.family.code.as_str()),
            "unexpected brown-like family: {}",
            result.short_id
        );
    }

    #[test]
    fn decode_encode_has_low_oklab_distance_error() {
        let registry = Registry::load_frozen().unwrap();
        let result = encode_from_hex("#E85A9A", &registry).unwrap();

        assert!(result.encoding_error < 1.0e-5, "{:?}", result);
    }

    #[test]
    fn encode_decode_returns_canonical_id() {
        let registry = Registry::load_frozen().unwrap();
        let id = OciId::parse_with_registry("OCI-1-46PK-236", &registry).unwrap();
        let decoded = decode_oci_id(&id, &registry).unwrap();
        let encoded = encode(ColorInput::Oklch(decoded), &registry).unwrap();

        assert_eq!(encoded.short_id, "OCI-1-46PK-236");
        assert_eq!(encoded.full_id, "OCI-1-46PK-A2-L12-C06");
    }

    #[test]
    fn short_and_full_ids_decode_to_same_base_step() {
        let registry = Registry::load_frozen().unwrap();
        let short = OciId::parse_with_registry("OCI-1-46PK-236", &registry).unwrap();
        let full = OciId::parse_with_registry("OCI-1-46PK-A2-L12-C06", &registry).unwrap();

        let short_oklch = decode_oci_id(&short, &registry).unwrap();
        let full_oklch = decode_oci_id(&full, &registry).unwrap();

        assert_close(short_oklch.l, full_oklch.l, EPS);
        assert_close(short_oklch.c, full_oklch.c, EPS);
        assert_close(short_oklch.h, full_oklch.h, EPS);
    }

    #[test]
    fn offset_id_decodes_correctly() {
        let registry = Registry::load_frozen().unwrap();
        let id = OciId::parse_with_registry(
            "OCI-1-46PK-236@L+0.002134,C-0.001042,H+0.218400",
            &registry,
        )
        .unwrap();
        let decoded = decode_oci_id(&id, &registry).unwrap();
        let base = registry.find_step(id.family, id.step).unwrap();

        assert_close(decoded.l, base.lightness + 0.002134, EPS);
        assert_close(decoded.c, base.chroma - 0.001042, EPS);
        assert_close(decoded.h, base.hue + 0.218400, EPS);
    }

    #[test]
    fn hue_wraparound_uses_shortest_offset() {
        let registry = Registry::load_frozen().unwrap();
        let id = OciId::parse_with_registry(
            "OCI-1-00RD-001@L+0.000000,C+0.000000,H-1.533333",
            &registry,
        )
        .unwrap();
        let decoded = decode_oci_id(&id, &registry).unwrap();

        assert_close(shortest_hue_diff(359.8, 1.333333), -1.533333, EPS);
        assert_close(decoded.h, 359.8, EPS);
    }

    #[test]
    fn candidate_family_search_uses_semantic_classes() {
        assert_eq!(
            candidate_family_indices(Oklch::new(0.67, 0.18, 355.3)),
            vec![48, 47, 49]
        );
        assert_eq!(
            candidate_family_indices(Oklch::new(0.02, 0.0, 0.0)),
            vec![63]
        );
        assert_eq!(
            candidate_family_indices(Oklch::new(0.98, 0.0, 0.0)),
            vec![62]
        );
        assert_eq!(
            candidate_family_indices(Oklch::new(0.45, 0.10, 35.0)),
            vec![52, 53, 54]
        );
    }

    #[test]
    fn tie_break_prefers_lower_chroma_level() {
        let registry = Registry::load_frozen().unwrap();
        let family = FamilyId::new(46, "PK").unwrap();
        let low = registry
            .find_step(family, StepId::new(2, 12, 5).unwrap())
            .unwrap();
        let high = registry
            .find_step(family, StepId::new(2, 12, 6).unwrap())
            .unwrap();
        let midpoint = Oklch::new(low.lightness, (low.chroma + high.chroma) / 2.0, low.hue);

        let result = encode(ColorInput::Oklch(midpoint), &registry).unwrap();
        assert_eq!(result.oci_id.family, family);
        assert_eq!(result.oci_id.step.anchor, 2);
        assert_eq!(result.oci_id.step.lightness, 12);
        assert_eq!(result.oci_id.step.chroma, 5);
    }

    #[test]
    fn export_support_statuses_include_rgb_like_spaces() {
        let matrix = build_support_matrix(EncodedSrgb::new(0.91, 0.35, 0.61).to_oklch());

        assert!(matrix.status_for(TargetColorSystem::SrgbFloat).is_some());
        assert_eq!(
            matrix.status_for(TargetColorSystem::Hex),
            Some(SupportStatus::Lossy)
        );
        assert!(
            matrix
                .status_for(TargetColorSystem::DisplayP3Float)
                .is_some()
        );
        assert!(
            matrix
                .status_for(TargetColorSystem::AdobeRgb1998Float)
                .is_some()
        );
        assert!(matrix.status_for(TargetColorSystem::Rec709Float).is_some());
    }

    #[test]
    fn out_of_gamut_colors_are_not_marked_supported() {
        let matrix = build_support_matrix(Oklch::new(0.5, 10.0, 140.0));

        assert_ne!(
            matrix.status_for(TargetColorSystem::SrgbFloat),
            Some(SupportStatus::Supported)
        );
        assert_ne!(
            matrix.status_for(TargetColorSystem::DisplayP3Float),
            Some(SupportStatus::Supported)
        );
        assert_ne!(
            matrix.status_for(TargetColorSystem::AdobeRgb1998Float),
            Some(SupportStatus::Supported)
        );
        assert_ne!(
            matrix.status_for(TargetColorSystem::Rec709Float),
            Some(SupportStatus::Supported)
        );
    }

    #[test]
    fn cmyk_support_matrix_entry_requires_profile() {
        let matrix = build_support_matrix(Oklch::new(0.6, 0.1, 30.0));
        assert_eq!(
            matrix.status_for(TargetColorSystem::Cmyk),
            Some(SupportStatus::ProfileRequired)
        );
    }

    #[test]
    fn inspect_base_id_returns_oklch_and_exports() {
        let registry = Registry::load_frozen().unwrap();
        let id = OciId::parse_with_registry("OCI-1-46PK-236", &registry).unwrap();
        let result = inspect(&id, &registry).unwrap();

        assert_eq!(result.short_id, "OCI-1-46PK-236");
        assert_eq!(result.full_id, "OCI-1-46PK-A2-L12-C06");
        assert_eq!(result.canonical_short_id, "OCI-1-46PK-236");
        assert!(!result.exports.json.is_empty());
        assert!(result.exports.css.oklch.starts_with("oklch("));
        assert!(
            result
                .support_matrix
                .status_for(TargetColorSystem::Hex)
                .is_some()
        );
    }

    #[test]
    fn all_required_input_forms_can_encode() {
        let registry = Registry::load_frozen().unwrap();
        let inputs = vec![
            ColorInput::SrgbRgb {
                r: 232,
                g: 90,
                b: 154,
            },
            ColorInput::HslSrgb {
                h: 333.0,
                s: 0.75,
                l: 0.63,
            },
            ColorInput::DisplayP3Float {
                r: 0.85,
                g: 0.40,
                b: 0.60,
            },
            ColorInput::AdobeRgb1998Float {
                r: 0.80,
                g: 0.45,
                b: 0.62,
            },
            ColorInput::Rec709Float {
                r: 0.82,
                g: 0.36,
                b: 0.58,
            },
            ColorInput::Oklab(Oklab::new(0.7, 0.12, -0.02)),
            ColorInput::OciId(OciId::parse_with_registry("OCI-1-46PK-236", &registry).unwrap()),
            ColorInput::OciIdString("OCI-1-46PK-236".to_string()),
        ];

        for input in inputs {
            let result = encode(input, &registry).unwrap();
            assert!(result.short_id.starts_with("OCI-1-"));
        }
    }
}
