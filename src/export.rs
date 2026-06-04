use crate::color::{
    EncodedAdobeRgb1998, EncodedDisplayP3, EncodedRec709, EncodedSrgb, Oklab, Oklch,
};
use crate::gamut::{
    SupportStatus, adobe_rgb_1998_status, display_p3_status, oklab_distance, rec709_status,
    srgb_status,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetColorSystem {
    SrgbFloat,
    Hex,
    Rgb8,
    HslSrgb,
    DisplayP3Float,
    AdobeRgb1998Float,
    Rec709Float,
    Oklch,
    Oklab,
    Css,
    Json,
    Cmyk,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatRgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsl {
    pub h: f64,
    pub s: f64,
    pub l: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorExport<T> {
    pub value: Option<T>,
    pub status: SupportStatus,
    pub round_trip_error: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CssColorSyntax {
    pub oklch: String,
    pub srgb: Option<String>,
    pub display_p3: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsonComponent {
    pub name: &'static str,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsonColorValue {
    pub model: &'static str,
    pub components: Vec<JsonComponent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportSet {
    pub srgb_float: ColorExport<FloatRgb>,
    pub hex: ColorExport<String>,
    pub rgb: ColorExport<Rgb8>,
    pub hsl: ColorExport<Hsl>,
    pub display_p3_float: ColorExport<FloatRgb>,
    pub adobe_rgb_1998_float: ColorExport<FloatRgb>,
    pub rec709_float: ColorExport<FloatRgb>,
    pub oklch: Oklch,
    pub oklab: Oklab,
    pub css: CssColorSyntax,
    pub json: Vec<JsonColorValue>,
    pub cmyk: ColorExport<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SupportEntry {
    pub target: TargetColorSystem,
    pub status: SupportStatus,
    pub round_trip_error: Option<f64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SupportMatrix {
    pub entries: Vec<SupportEntry>,
}

impl SupportMatrix {
    #[must_use]
    pub fn status_for(&self, target: TargetColorSystem) -> Option<SupportStatus> {
        self.entries
            .iter()
            .find(|entry| entry.target == target)
            .map(|entry| entry.status)
    }

    #[must_use]
    pub fn entry_for(&self, target: TargetColorSystem) -> Option<&SupportEntry> {
        self.entries.iter().find(|entry| entry.target == target)
    }
}

#[must_use]
pub fn export_all(color: Oklch) -> ExportSet {
    let canonical = normalize_oklch(color);
    let matrix = build_support_matrix(canonical);

    let srgb = canonical.to_encoded_srgb();
    let srgb_entry = matrix
        .entry_for(TargetColorSystem::SrgbFloat)
        .expect("support matrix includes sRGB");
    let srgb_float = if srgb_entry.status == SupportStatus::Supported {
        ColorExport {
            value: Some(FloatRgb::from(srgb)),
            status: srgb_entry.status,
            round_trip_error: srgb_entry.round_trip_error,
        }
    } else {
        ColorExport {
            value: None,
            status: srgb_entry.status,
            round_trip_error: srgb_entry.round_trip_error,
        }
    };

    let hex_entry = matrix
        .entry_for(TargetColorSystem::Hex)
        .expect("support matrix includes HEX");
    let hex = ColorExport {
        value: (hex_entry.status == SupportStatus::Lossy).then(|| srgb_to_hex(srgb)),
        status: hex_entry.status,
        round_trip_error: hex_entry.round_trip_error,
    };

    let rgb_entry = matrix
        .entry_for(TargetColorSystem::Rgb8)
        .expect("support matrix includes RGB8");
    let rgb = ColorExport {
        value: (rgb_entry.status == SupportStatus::Lossy).then(|| srgb_to_rgb8(srgb)),
        status: rgb_entry.status,
        round_trip_error: rgb_entry.round_trip_error,
    };

    let hsl_entry = matrix
        .entry_for(TargetColorSystem::HslSrgb)
        .expect("support matrix includes HSL");
    let hsl = ColorExport {
        value: (hsl_entry.status == SupportStatus::Supported).then(|| srgb_to_hsl(srgb)),
        status: hsl_entry.status,
        round_trip_error: hsl_entry.round_trip_error,
    };

    let display_p3 = canonical.to_encoded_display_p3();
    let display_p3_entry = matrix
        .entry_for(TargetColorSystem::DisplayP3Float)
        .expect("support matrix includes Display P3");
    let display_p3_float = ColorExport {
        value: (display_p3_entry.status == SupportStatus::Supported)
            .then(|| FloatRgb::from(display_p3)),
        status: display_p3_entry.status,
        round_trip_error: display_p3_entry.round_trip_error,
    };

    let adobe = canonical.to_encoded_adobe_rgb_1998();
    let adobe_entry = matrix
        .entry_for(TargetColorSystem::AdobeRgb1998Float)
        .expect("support matrix includes Adobe RGB");
    let adobe_rgb_1998_float = ColorExport {
        value: (adobe_entry.status == SupportStatus::Supported).then(|| FloatRgb::from(adobe)),
        status: adobe_entry.status,
        round_trip_error: adobe_entry.round_trip_error,
    };

    let rec709 = canonical.to_encoded_rec709();
    let rec709_entry = matrix
        .entry_for(TargetColorSystem::Rec709Float)
        .expect("support matrix includes Rec.709");
    let rec709_float = ColorExport {
        value: (rec709_entry.status == SupportStatus::Supported).then(|| FloatRgb::from(rec709)),
        status: rec709_entry.status,
        round_trip_error: rec709_entry.round_trip_error,
    };

    ExportSet {
        srgb_float,
        hex,
        rgb,
        hsl,
        display_p3_float,
        adobe_rgb_1998_float,
        rec709_float,
        oklch: canonical,
        oklab: canonical.to_oklab(),
        css: CssColorSyntax {
            oklch: format!(
                "oklch({:.6}% {:.6} {:.6}deg)",
                canonical.l * 100.0,
                canonical.c,
                canonical.h
            ),
            srgb: srgb.is_in_gamut().then(|| {
                let rgb = srgb_to_rgb8(srgb);
                format!("rgb({} {} {})", rgb.r, rgb.g, rgb.b)
            }),
            display_p3: display_p3.is_in_gamut().then(|| {
                format!(
                    "color(display-p3 {:.6} {:.6} {:.6})",
                    display_p3.r, display_p3.g, display_p3.b
                )
            }),
        },
        json: vec![
            JsonColorValue {
                model: "oklch",
                components: vec![
                    JsonComponent {
                        name: "l",
                        value: canonical.l,
                    },
                    JsonComponent {
                        name: "c",
                        value: canonical.c,
                    },
                    JsonComponent {
                        name: "h",
                        value: canonical.h,
                    },
                ],
            },
            JsonColorValue {
                model: "oklab",
                components: vec![
                    JsonComponent {
                        name: "l",
                        value: canonical.to_oklab().l,
                    },
                    JsonComponent {
                        name: "a",
                        value: canonical.to_oklab().a,
                    },
                    JsonComponent {
                        name: "b",
                        value: canonical.to_oklab().b,
                    },
                ],
            },
        ],
        cmyk: ColorExport {
            value: None,
            status: SupportStatus::ProfileRequired,
            round_trip_error: None,
        },
    }
}

#[must_use]
pub fn build_support_matrix(color: Oklch) -> SupportMatrix {
    let canonical = normalize_oklch(color);
    let mut entries = Vec::new();

    let srgb = canonical.to_encoded_srgb();
    entries.push(rgb_support_entry(
        TargetColorSystem::SrgbFloat,
        srgb_status(srgb),
        canonical,
        || srgb.to_oklch(),
        None,
    ));
    entries.push(hex_support_entry(canonical, srgb));
    entries.push(rgb8_support_entry(canonical, srgb));
    entries.push(hsl_support_entry(canonical, srgb));

    let p3 = canonical.to_encoded_display_p3();
    entries.push(rgb_support_entry(
        TargetColorSystem::DisplayP3Float,
        display_p3_status(p3),
        canonical,
        || p3.to_oklch(),
        None,
    ));

    let adobe = canonical.to_encoded_adobe_rgb_1998();
    entries.push(rgb_support_entry(
        TargetColorSystem::AdobeRgb1998Float,
        adobe_rgb_1998_status(adobe),
        canonical,
        || adobe.to_oklch(),
        None,
    ));

    let rec709 = canonical.to_encoded_rec709();
    entries.push(rgb_support_entry(
        TargetColorSystem::Rec709Float,
        rec709_status(rec709),
        canonical,
        || rec709.to_oklch(),
        None,
    ));

    entries.push(SupportEntry {
        target: TargetColorSystem::Oklch,
        status: SupportStatus::Supported,
        round_trip_error: Some(0.0),
        note: None,
    });
    entries.push(SupportEntry {
        target: TargetColorSystem::Oklab,
        status: SupportStatus::Supported,
        round_trip_error: Some(0.0),
        note: None,
    });
    entries.push(SupportEntry {
        target: TargetColorSystem::Css,
        status: SupportStatus::Supported,
        round_trip_error: None,
        note: Some("CSS OKLCH syntax is emitted; RGB syntax is emitted only when in gamut".into()),
    });
    entries.push(SupportEntry {
        target: TargetColorSystem::Json,
        status: SupportStatus::Supported,
        round_trip_error: None,
        note: Some("Plain data structs are JSON-friendly; serialization is caller-owned".into()),
    });
    entries.push(SupportEntry {
        target: TargetColorSystem::Cmyk,
        status: SupportStatus::ProfileRequired,
        round_trip_error: None,
        note: Some("CMYK requires an ICC/profile workflow and is not numeric in v1-beta".into()),
    });

    SupportMatrix { entries }
}

impl From<EncodedSrgb> for FloatRgb {
    fn from(value: EncodedSrgb) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
        }
    }
}

impl From<EncodedDisplayP3> for FloatRgb {
    fn from(value: EncodedDisplayP3) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
        }
    }
}

impl From<EncodedAdobeRgb1998> for FloatRgb {
    fn from(value: EncodedAdobeRgb1998) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
        }
    }
}

impl From<EncodedRec709> for FloatRgb {
    fn from(value: EncodedRec709) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
        }
    }
}

fn rgb_support_entry(
    target: TargetColorSystem,
    status: SupportStatus,
    source: Oklch,
    round_trip: impl FnOnce() -> Oklch,
    note: Option<String>,
) -> SupportEntry {
    let round_trip_error = (status == SupportStatus::Supported)
        .then(|| oklab_distance(source.to_oklab(), round_trip().to_oklab()));
    SupportEntry {
        target,
        status,
        round_trip_error,
        note,
    }
}

fn hex_support_entry(source: Oklch, srgb: EncodedSrgb) -> SupportEntry {
    if !srgb.is_in_gamut() {
        return SupportEntry {
            target: TargetColorSystem::Hex,
            status: SupportStatus::Unsupported,
            round_trip_error: None,
            note: Some("HEX is 8-bit sRGB and the color is outside sRGB gamut".into()),
        };
    }

    let quantized = rgb8_to_srgb(srgb_to_rgb8(srgb));
    SupportEntry {
        target: TargetColorSystem::Hex,
        status: SupportStatus::Lossy,
        round_trip_error: Some(oklab_distance(
            source.to_oklab(),
            quantized.to_oklch().to_oklab(),
        )),
        note: Some("HEX is always lossy because it quantizes to 8-bit sRGB".into()),
    }
}

fn rgb8_support_entry(source: Oklch, srgb: EncodedSrgb) -> SupportEntry {
    if !srgb.is_in_gamut() {
        return SupportEntry {
            target: TargetColorSystem::Rgb8,
            status: SupportStatus::Unsupported,
            round_trip_error: None,
            note: Some("8-bit RGB is sRGB-bound and the color is outside sRGB gamut".into()),
        };
    }

    let quantized = rgb8_to_srgb(srgb_to_rgb8(srgb));
    SupportEntry {
        target: TargetColorSystem::Rgb8,
        status: SupportStatus::Lossy,
        round_trip_error: Some(oklab_distance(
            source.to_oklab(),
            quantized.to_oklch().to_oklab(),
        )),
        note: Some("RGB8 quantizes to 8-bit sRGB".into()),
    }
}

fn hsl_support_entry(source: Oklch, srgb: EncodedSrgb) -> SupportEntry {
    if !srgb.is_in_gamut() {
        return SupportEntry {
            target: TargetColorSystem::HslSrgb,
            status: SupportStatus::Unsupported,
            round_trip_error: None,
            note: Some(
                "HSL here is an sRGB representation and the color is outside sRGB gamut".into(),
            ),
        };
    }

    let hsl = srgb_to_hsl(srgb);
    let round_trip = EncodedSrgb::from_hsl(hsl.h, hsl.s, hsl.l)
        .expect("HSL export produces valid sRGB HSL")
        .to_oklch();
    SupportEntry {
        target: TargetColorSystem::HslSrgb,
        status: SupportStatus::Supported,
        round_trip_error: Some(oklab_distance(source.to_oklab(), round_trip.to_oklab())),
        note: None,
    }
}

#[must_use]
pub fn srgb_to_hex(srgb: EncodedSrgb) -> String {
    let rgb = srgb_to_rgb8(srgb);
    format!("#{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b)
}

#[must_use]
pub fn srgb_to_rgb8(srgb: EncodedSrgb) -> Rgb8 {
    Rgb8 {
        r: quantize_u8(srgb.r),
        g: quantize_u8(srgb.g),
        b: quantize_u8(srgb.b),
    }
}

#[must_use]
pub fn rgb8_to_srgb(rgb: Rgb8) -> EncodedSrgb {
    EncodedSrgb::new(
        f64::from(rgb.r) / 255.0,
        f64::from(rgb.g) / 255.0,
        f64::from(rgb.b) / 255.0,
    )
}

#[must_use]
pub fn srgb_to_hsl(srgb: EncodedSrgb) -> Hsl {
    let max = srgb.r.max(srgb.g).max(srgb.b);
    let min = srgb.r.min(srgb.g).min(srgb.b);
    let l = (max + min) / 2.0;
    let delta = max - min;

    if delta.abs() < 1.0e-12 {
        return Hsl { h: 0.0, s: 0.0, l };
    }

    let s = if l > 0.5 {
        delta / (2.0 - max - min)
    } else {
        delta / (max + min)
    };

    let h = if (max - srgb.r).abs() < 1.0e-12 {
        60.0 * (((srgb.g - srgb.b) / delta).rem_euclid(6.0))
    } else if (max - srgb.g).abs() < 1.0e-12 {
        60.0 * (((srgb.b - srgb.r) / delta) + 2.0)
    } else {
        60.0 * (((srgb.r - srgb.g) / delta) + 4.0)
    };

    Hsl {
        h: h.rem_euclid(360.0),
        s,
        l,
    }
}

#[must_use]
pub fn normalize_oklch(color: Oklch) -> Oklch {
    let mut l = color.l;
    let mut c = color.c;
    let mut h = color.h;

    if c < 0.0 {
        c = -c;
        h += 180.0;
    }
    if c.abs() < 1.0e-12 {
        c = 0.0;
        h = 0.0;
    }

    if !l.is_finite() {
        l = 0.0;
    }
    if !c.is_finite() {
        c = 0.0;
    }
    if !h.is_finite() {
        h = 0.0;
    }

    Oklch {
        l,
        c,
        h: h.rem_euclid(360.0),
    }
}

fn quantize_u8(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_is_lossy_for_in_gamut_colors() {
        let matrix = build_support_matrix(EncodedSrgb::new(0.91, 0.35, 0.61).to_oklch());
        assert_eq!(
            matrix.status_for(TargetColorSystem::Hex),
            Some(SupportStatus::Lossy)
        );
    }

    #[test]
    fn cmyk_requires_profile() {
        let matrix = build_support_matrix(Oklch::new(0.6, 0.1, 30.0));
        assert_eq!(
            matrix.status_for(TargetColorSystem::Cmyk),
            Some(SupportStatus::ProfileRequired)
        );
    }
}
