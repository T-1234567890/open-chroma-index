use crate::color::{
    EncodedAdobeRgb1998, EncodedDisplayP3, EncodedRec709, EncodedSrgb, LinearAdobeRgb1998,
    LinearDisplayP3, LinearRec709, LinearSrgb, Oklab, Oklch, XyzD65,
};

const ADOBE_RGB_1998_GAMMA: f64 = 563.0 / 256.0;

type MatrixRow = (f64, f64, f64);
type MatrixRows = (MatrixRow, MatrixRow, MatrixRow);

#[derive(Debug, Clone, Copy)]
struct Matrix3 {
    m11: f64,
    m12: f64,
    m13: f64,
    m21: f64,
    m22: f64,
    m23: f64,
    m31: f64,
    m32: f64,
    m33: f64,
}

impl Matrix3 {
    const fn new(rows: MatrixRows) -> Self {
        Self {
            m11: rows.0.0,
            m12: rows.0.1,
            m13: rows.0.2,
            m21: rows.1.0,
            m22: rows.1.1,
            m23: rows.1.2,
            m31: rows.2.0,
            m32: rows.2.1,
            m33: rows.2.2,
        }
    }

    fn rgb_to_xyz(self, r: f64, g: f64, b: f64) -> XyzD65 {
        XyzD65 {
            x: self.m11 * r + self.m12 * g + self.m13 * b,
            y: self.m21 * r + self.m22 * g + self.m23 * b,
            z: self.m31 * r + self.m32 * g + self.m33 * b,
        }
    }

    fn xyz_to_rgb(self, xyz: XyzD65) -> (f64, f64, f64) {
        (
            self.m11 * xyz.x + self.m12 * xyz.y + self.m13 * xyz.z,
            self.m21 * xyz.x + self.m22 * xyz.y + self.m23 * xyz.z,
            self.m31 * xyz.x + self.m32 * xyz.y + self.m33 * xyz.z,
        )
    }
}

const SRGB_TO_XYZ: Matrix3 = Matrix3::new((
    (0.412_456_4, 0.357_576_1, 0.180_437_5),
    (0.212_672_9, 0.715_152_2, 0.072_175),
    (0.019_333_9, 0.119_192, 0.950_304_1),
));

const XYZ_TO_SRGB: Matrix3 = Matrix3::new((
    (3.240_454_2, -1.537_138_5, -0.498_531_4),
    (-0.969_266, 1.876_010_8, 0.041_556),
    (0.055_643_4, -0.204_025_9, 1.057_225_2),
));

const DISPLAY_P3_TO_XYZ: Matrix3 = Matrix3::new((
    (
        0.486_570_948_648_216_2,
        0.265_667_693_169_093_06,
        0.198_217_285_234_362_5,
    ),
    (
        0.228_974_564_069_748_8,
        0.691_738_521_836_506_4,
        0.079_286_914_093_745,
    ),
    (0.0, 0.045_113_381_858_902_64, 1.043_944_368_900_976),
));

const XYZ_TO_DISPLAY_P3: Matrix3 = Matrix3::new((
    (
        2.493_496_911_941_425,
        -0.931_383_617_919_123_9,
        -0.402_710_784_450_716_84,
    ),
    (
        -0.829_488_969_561_574_7,
        1.762_664_060_318_346_3,
        0.023_624_685_841_943_577,
    ),
    (
        0.035_845_830_243_784_47,
        -0.076_172_389_268_041_82,
        0.956_884_524_007_687_2,
    ),
));

const ADOBE_RGB_1998_TO_XYZ: Matrix3 = Matrix3::new((
    (0.576_730_9, 0.185_554, 0.188_185_2),
    (0.297_376_9, 0.627_349_1, 0.075_274_1),
    (0.027_034_3, 0.070_687_2, 0.991_108_5),
));

const XYZ_TO_ADOBE_RGB_1998: Matrix3 = Matrix3::new((
    (2.041_369, -0.564_946_4, -0.344_694_4),
    (-0.969_266, 1.876_010_8, 0.041_556),
    (0.013_447_4, -0.118_389_7, 1.015_409_6),
));

impl EncodedSrgb {
    #[must_use]
    pub fn to_linear(self) -> LinearSrgb {
        LinearSrgb {
            r: srgb_decode(self.r),
            g: srgb_decode(self.g),
            b: srgb_decode(self.b),
        }
    }

    #[must_use]
    pub fn to_xyz_d65(self) -> XyzD65 {
        self.to_linear().to_xyz_d65()
    }

    #[must_use]
    pub fn to_oklab(self) -> Oklab {
        self.to_xyz_d65().to_oklab()
    }

    #[must_use]
    pub fn to_oklch(self) -> Oklch {
        self.to_oklab().to_oklch()
    }
}

impl LinearSrgb {
    #[must_use]
    pub fn to_encoded(self) -> EncodedSrgb {
        EncodedSrgb {
            r: srgb_encode(self.r),
            g: srgb_encode(self.g),
            b: srgb_encode(self.b),
        }
    }

    #[must_use]
    pub fn to_xyz_d65(self) -> XyzD65 {
        SRGB_TO_XYZ.rgb_to_xyz(self.r, self.g, self.b)
    }
}

impl EncodedDisplayP3 {
    #[must_use]
    pub fn to_linear(self) -> LinearDisplayP3 {
        LinearDisplayP3 {
            r: srgb_decode(self.r),
            g: srgb_decode(self.g),
            b: srgb_decode(self.b),
        }
    }

    #[must_use]
    pub fn to_xyz_d65(self) -> XyzD65 {
        self.to_linear().to_xyz_d65()
    }

    #[must_use]
    pub fn to_oklab(self) -> Oklab {
        self.to_xyz_d65().to_oklab()
    }

    #[must_use]
    pub fn to_oklch(self) -> Oklch {
        self.to_oklab().to_oklch()
    }
}

impl LinearDisplayP3 {
    #[must_use]
    pub fn to_encoded(self) -> EncodedDisplayP3 {
        EncodedDisplayP3 {
            r: srgb_encode(self.r),
            g: srgb_encode(self.g),
            b: srgb_encode(self.b),
        }
    }

    #[must_use]
    pub fn to_xyz_d65(self) -> XyzD65 {
        DISPLAY_P3_TO_XYZ.rgb_to_xyz(self.r, self.g, self.b)
    }
}

impl EncodedAdobeRgb1998 {
    #[must_use]
    pub fn to_linear(self) -> LinearAdobeRgb1998 {
        LinearAdobeRgb1998 {
            r: signed_pow(self.r, ADOBE_RGB_1998_GAMMA),
            g: signed_pow(self.g, ADOBE_RGB_1998_GAMMA),
            b: signed_pow(self.b, ADOBE_RGB_1998_GAMMA),
        }
    }

    #[must_use]
    pub fn to_xyz_d65(self) -> XyzD65 {
        self.to_linear().to_xyz_d65()
    }

    #[must_use]
    pub fn to_oklab(self) -> Oklab {
        self.to_xyz_d65().to_oklab()
    }

    #[must_use]
    pub fn to_oklch(self) -> Oklch {
        self.to_oklab().to_oklch()
    }
}

impl LinearAdobeRgb1998 {
    #[must_use]
    pub fn to_encoded(self) -> EncodedAdobeRgb1998 {
        EncodedAdobeRgb1998 {
            r: signed_pow(self.r, 1.0 / ADOBE_RGB_1998_GAMMA),
            g: signed_pow(self.g, 1.0 / ADOBE_RGB_1998_GAMMA),
            b: signed_pow(self.b, 1.0 / ADOBE_RGB_1998_GAMMA),
        }
    }

    #[must_use]
    pub fn to_xyz_d65(self) -> XyzD65 {
        ADOBE_RGB_1998_TO_XYZ.rgb_to_xyz(self.r, self.g, self.b)
    }
}

impl EncodedRec709 {
    #[must_use]
    pub fn to_linear(self) -> LinearRec709 {
        LinearRec709 {
            r: rec709_decode(self.r),
            g: rec709_decode(self.g),
            b: rec709_decode(self.b),
        }
    }

    #[must_use]
    pub fn to_xyz_d65(self) -> XyzD65 {
        self.to_linear().to_xyz_d65()
    }

    #[must_use]
    pub fn to_oklab(self) -> Oklab {
        self.to_xyz_d65().to_oklab()
    }

    #[must_use]
    pub fn to_oklch(self) -> Oklch {
        self.to_oklab().to_oklch()
    }
}

impl LinearRec709 {
    #[must_use]
    pub fn to_encoded(self) -> EncodedRec709 {
        EncodedRec709 {
            r: rec709_encode(self.r),
            g: rec709_encode(self.g),
            b: rec709_encode(self.b),
        }
    }

    #[must_use]
    pub fn to_xyz_d65(self) -> XyzD65 {
        SRGB_TO_XYZ.rgb_to_xyz(self.r, self.g, self.b)
    }
}

impl XyzD65 {
    #[must_use]
    pub fn to_linear_srgb(self) -> LinearSrgb {
        let (r, g, b) = XYZ_TO_SRGB.xyz_to_rgb(self);
        LinearSrgb { r, g, b }
    }

    #[must_use]
    pub fn to_encoded_srgb(self) -> EncodedSrgb {
        self.to_linear_srgb().to_encoded()
    }

    #[must_use]
    pub fn to_linear_display_p3(self) -> LinearDisplayP3 {
        let (r, g, b) = XYZ_TO_DISPLAY_P3.xyz_to_rgb(self);
        LinearDisplayP3 { r, g, b }
    }

    #[must_use]
    pub fn to_encoded_display_p3(self) -> EncodedDisplayP3 {
        self.to_linear_display_p3().to_encoded()
    }

    #[must_use]
    pub fn to_linear_adobe_rgb_1998(self) -> LinearAdobeRgb1998 {
        let (r, g, b) = XYZ_TO_ADOBE_RGB_1998.xyz_to_rgb(self);
        LinearAdobeRgb1998 { r, g, b }
    }

    #[must_use]
    pub fn to_encoded_adobe_rgb_1998(self) -> EncodedAdobeRgb1998 {
        self.to_linear_adobe_rgb_1998().to_encoded()
    }

    #[must_use]
    pub fn to_linear_rec709(self) -> LinearRec709 {
        let (r, g, b) = XYZ_TO_SRGB.xyz_to_rgb(self);
        LinearRec709 { r, g, b }
    }

    #[must_use]
    pub fn to_encoded_rec709(self) -> EncodedRec709 {
        self.to_linear_rec709().to_encoded()
    }

    #[must_use]
    pub fn to_oklab(self) -> Oklab {
        let l = 0.818_933_010_1 * self.x + 0.361_866_742_4 * self.y - 0.128_859_713_7 * self.z;
        let m = 0.032_984_543_6 * self.x + 0.929_311_871_5 * self.y + 0.036_145_638_7 * self.z;
        let s = 0.048_200_301_8 * self.x + 0.264_366_269_1 * self.y + 0.633_851_707 * self.z;

        let l_ = l.cbrt();
        let m_ = m.cbrt();
        let s_ = s.cbrt();

        Oklab {
            l: 0.210_454_255_3 * l_ + 0.793_617_785 * m_ - 0.004_072_046_8 * s_,
            a: 1.977_998_495_1 * l_ - 2.428_592_205 * m_ + 0.450_593_709_9 * s_,
            b: 0.025_904_037_1 * l_ + 0.782_771_766_2 * m_ - 0.808_675_766 * s_,
        }
    }
}

impl Oklab {
    #[must_use]
    pub fn to_xyz_d65(self) -> XyzD65 {
        let l_ = self.l + 0.396_337_777_4 * self.a + 0.215_803_757_3 * self.b;
        let m_ = self.l - 0.105_561_345_8 * self.a - 0.063_854_172_8 * self.b;
        let s_ = self.l - 0.089_484_177_5 * self.a - 1.291_485_548 * self.b;

        let l = l_ * l_ * l_;
        let m = m_ * m_ * m_;
        let s = s_ * s_ * s_;

        XyzD65 {
            x: 1.227_013_851_1 * l - 0.557_799_980_7 * m + 0.281_256_149 * s,
            y: -0.040_580_178_4 * l + 1.112_256_869_6 * m - 0.071_676_678_7 * s,
            z: -0.076_381_284_5 * l - 0.421_481_978_4 * m + 1.586_163_220_4 * s,
        }
    }

    #[must_use]
    pub fn to_oklch(self) -> Oklch {
        let c = self.a.hypot(self.b);
        let h = self.b.atan2(self.a).to_degrees().rem_euclid(360.0);

        Oklch { l: self.l, c, h }
    }
}

impl Oklch {
    #[must_use]
    pub fn to_oklab(self) -> Oklab {
        let hue = self.h.rem_euclid(360.0).to_radians();

        Oklab {
            l: self.l,
            a: self.c * hue.cos(),
            b: self.c * hue.sin(),
        }
    }

    #[must_use]
    pub fn to_xyz_d65(self) -> XyzD65 {
        self.to_oklab().to_xyz_d65()
    }

    #[must_use]
    pub fn to_encoded_srgb(self) -> EncodedSrgb {
        self.to_xyz_d65().to_encoded_srgb()
    }

    #[must_use]
    pub fn to_encoded_display_p3(self) -> EncodedDisplayP3 {
        self.to_xyz_d65().to_encoded_display_p3()
    }

    #[must_use]
    pub fn to_encoded_adobe_rgb_1998(self) -> EncodedAdobeRgb1998 {
        self.to_xyz_d65().to_encoded_adobe_rgb_1998()
    }

    #[must_use]
    pub fn to_encoded_rec709(self) -> EncodedRec709 {
        self.to_xyz_d65().to_encoded_rec709()
    }
}

#[must_use]
pub fn srgb_to_oklch_to_srgb(color: EncodedSrgb) -> EncodedSrgb {
    color.to_oklch().to_encoded_srgb()
}

#[must_use]
pub fn display_p3_to_oklch_to_display_p3(color: EncodedDisplayP3) -> EncodedDisplayP3 {
    color.to_oklch().to_encoded_display_p3()
}

#[must_use]
pub fn adobe_rgb_1998_to_oklch_to_adobe_rgb_1998(
    color: EncodedAdobeRgb1998,
) -> EncodedAdobeRgb1998 {
    color.to_oklch().to_encoded_adobe_rgb_1998()
}

#[must_use]
pub fn rec709_to_oklch_to_rec709(color: EncodedRec709) -> EncodedRec709 {
    color.to_oklch().to_encoded_rec709()
}

#[must_use]
pub fn srgb_decode(channel: f64) -> f64 {
    if channel <= 0.040_45 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

#[must_use]
pub fn srgb_encode(channel: f64) -> f64 {
    if channel <= 0.003_130_8 {
        12.92 * channel
    } else {
        1.055 * channel.powf(1.0 / 2.4) - 0.055
    }
}

#[must_use]
pub fn rec709_decode(channel: f64) -> f64 {
    if channel < 0.081 {
        channel / 4.5
    } else {
        ((channel + 0.099) / 1.099).powf(1.0 / 0.45)
    }
}

#[must_use]
pub fn rec709_encode(channel: f64) -> f64 {
    if channel < 0.018 {
        4.5 * channel
    } else {
        1.099 * channel.powf(0.45) - 0.099
    }
}

fn signed_pow(value: f64, power: f64) -> f64 {
    if value < 0.0 {
        -(-value).powf(power)
    } else {
        value.powf(power)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ColorError;

    const EPS: f64 = 1.0e-6;

    fn assert_close(actual: f64, expected: f64, epsilon: f64) {
        assert!(
            (actual - expected).abs() <= epsilon,
            "expected {actual} to be within {epsilon} of {expected}"
        );
    }

    fn assert_rgb_close(actual: (f64, f64, f64), expected: (f64, f64, f64), epsilon: f64) {
        assert_close(actual.0, expected.0, epsilon);
        assert_close(actual.1, expected.1, epsilon);
        assert_close(actual.2, expected.2, epsilon);
    }

    #[test]
    fn parses_hex_to_encoded_srgb() {
        let magenta = EncodedSrgb::from_hex("#ff00aa").unwrap();
        assert_rgb_close(
            (magenta.r, magenta.g, magenta.b),
            (1.0, 0.0, 170.0 / 255.0),
            EPS,
        );

        let short = EncodedSrgb::from_hex("0f8").unwrap();
        assert_rgb_close((short.r, short.g, short.b), (0.0, 1.0, 136.0 / 255.0), EPS);

        assert_eq!(
            EncodedSrgb::from_hex("#abcd").unwrap_err(),
            ColorError::InvalidHexLength { found: 4 }
        );
        assert_eq!(
            EncodedSrgb::from_hex("#zzzzzz").unwrap_err(),
            ColorError::InvalidHexDigit
        );
    }

    #[test]
    fn converts_hsl_to_encoded_srgb() {
        let red = EncodedSrgb::from_hsl(0.0, 1.0, 0.5).unwrap();
        assert_rgb_close((red.r, red.g, red.b), (1.0, 0.0, 0.0), EPS);

        let wrapped = EncodedSrgb::from_hsl(480.0, 1.0, 0.25).unwrap();
        assert_rgb_close((wrapped.r, wrapped.g, wrapped.b), (0.0, 0.5, 0.0), EPS);
    }

    #[test]
    fn linearizes_and_encodes_srgb_channels() {
        assert_close(srgb_decode(0.040_45), 0.003_130_804_953_560_371_8, 1.0e-12);
        assert_close(srgb_decode(1.0), 1.0, 1.0e-12);
        assert_close(srgb_encode(0.003_130_8), 0.040_449_936, 1.0e-9);
        assert_close(srgb_encode(1.0), 1.0, 1.0e-12);

        let encoded = EncodedSrgb::new(0.25, 0.5, 0.75);
        let round_trip = encoded.to_linear().to_encoded();
        assert_rgb_close(
            (round_trip.r, round_trip.g, round_trip.b),
            (encoded.r, encoded.g, encoded.b),
            EPS,
        );
    }

    #[test]
    fn round_trips_srgb_through_oklch() {
        let color = EncodedSrgb::new(0.2, 0.5, 0.9);
        let round_trip = srgb_to_oklch_to_srgb(color);
        assert_rgb_close(
            (round_trip.r, round_trip.g, round_trip.b),
            (color.r, color.g, color.b),
            EPS,
        );
    }

    #[test]
    fn round_trips_display_p3_through_oklch() {
        let color = EncodedDisplayP3::new(0.9, 0.35, 0.2);
        let round_trip = display_p3_to_oklch_to_display_p3(color);
        assert_rgb_close(
            (round_trip.r, round_trip.g, round_trip.b),
            (color.r, color.g, color.b),
            EPS,
        );
    }

    #[test]
    fn round_trips_adobe_rgb_1998_through_oklch() {
        let color = EncodedAdobeRgb1998::new(0.1, 0.8, 0.45);
        let round_trip = adobe_rgb_1998_to_oklch_to_adobe_rgb_1998(color);
        assert_rgb_close(
            (round_trip.r, round_trip.g, round_trip.b),
            (color.r, color.g, color.b),
            EPS,
        );
    }

    #[test]
    fn round_trips_rec709_through_oklch() {
        let color = EncodedRec709::new(0.75, 0.25, 0.4);
        let round_trip = rec709_to_oklch_to_rec709(color);
        assert_rgb_close(
            (round_trip.r, round_trip.g, round_trip.b),
            (color.r, color.g, color.b),
            EPS,
        );
    }

    #[test]
    fn converts_oklab_to_oklch_and_back() {
        let lab = Oklab::new(0.62, -0.11, 0.18);
        let lch = lab.to_oklch();
        let round_trip = lch.to_oklab();

        assert_close(lch.l, 0.62, EPS);
        assert_close(lch.c, (-0.11_f64).hypot(0.18), EPS);
        assert_close(round_trip.l, lab.l, EPS);
        assert_close(round_trip.a, lab.a, EPS);
        assert_close(round_trip.b, lab.b, EPS);
    }

    #[test]
    fn checks_rgb_gamut() {
        assert!(EncodedSrgb::new(0.0, 0.5, 1.0).is_in_gamut());
        assert!(LinearDisplayP3::new(1.0, 0.0, 0.25).is_in_gamut());
        assert!(!EncodedAdobeRgb1998::new(-0.01, 0.5, 0.5).is_in_gamut());
        assert!(!LinearRec709::new(0.0, 1.01, 0.0).is_in_gamut());
    }

    #[test]
    fn wraps_oklch_hue() {
        let from_negative = Oklch::new(0.5, 0.2, -30.0).to_oklab().to_oklch();
        let from_large = Oklch::new(0.5, 0.2, 390.0).to_oklab().to_oklch();

        assert_close(from_negative.h, 330.0, EPS);
        assert_close(from_large.h, 30.0, EPS);
    }
}
