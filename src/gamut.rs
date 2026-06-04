use crate::color::{EncodedAdobeRgb1998, EncodedDisplayP3, EncodedRec709, EncodedSrgb, Oklab};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportStatus {
    Supported,
    Lossy,
    GamutMapped,
    Approximation,
    Unsupported,
    ProfileRequired,
    ProofRequired,
    UserSuppliedReference,
}

#[must_use]
pub fn oklab_distance(a: Oklab, b: Oklab) -> f64 {
    ((a.l - b.l).powi(2) + (a.a - b.a).powi(2) + (a.b - b.b).powi(2)).sqrt()
}

#[must_use]
pub fn shortest_hue_diff(target_hue: f64, base_hue: f64) -> f64 {
    let diff = (target_hue - base_hue + 180.0).rem_euclid(360.0) - 180.0;
    if diff == -180.0 { 180.0 } else { diff }
}

#[must_use]
pub fn srgb_status(color: EncodedSrgb) -> SupportStatus {
    if color.is_in_gamut() {
        SupportStatus::Supported
    } else {
        SupportStatus::Unsupported
    }
}

#[must_use]
pub fn display_p3_status(color: EncodedDisplayP3) -> SupportStatus {
    if color.is_in_gamut() {
        SupportStatus::Supported
    } else {
        SupportStatus::Unsupported
    }
}

#[must_use]
pub fn adobe_rgb_1998_status(color: EncodedAdobeRgb1998) -> SupportStatus {
    if color.is_in_gamut() {
        SupportStatus::Supported
    } else {
        SupportStatus::Unsupported
    }
}

#[must_use]
pub fn rec709_status(color: EncodedRec709) -> SupportStatus {
    if color.is_in_gamut() {
        SupportStatus::Supported
    } else {
        SupportStatus::Unsupported
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hue_diff_wraps_across_zero() {
        assert!((shortest_hue_diff(1.0, 359.0) - 2.0).abs() < 1.0e-12);
        assert!((shortest_hue_diff(359.0, 1.0) + 2.0).abs() < 1.0e-12);
    }
}
