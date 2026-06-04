use crate::error::ColorError;

const GAMUT_EPSILON: f64 = 1.0e-12;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EncodedSrgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearSrgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EncodedDisplayP3 {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearDisplayP3 {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EncodedAdobeRgb1998 {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearAdobeRgb1998 {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EncodedRec709 {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearRec709 {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XyzD65 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklch {
    pub l: f64,
    pub c: f64,
    pub h: f64,
}

macro_rules! rgb_impl {
    ($name:ident) => {
        impl $name {
            #[must_use]
            pub const fn new(r: f64, g: f64, b: f64) -> Self {
                Self { r, g, b }
            }

            #[must_use]
            pub fn is_in_gamut(self) -> bool {
                channel_in_unit_interval(self.r)
                    && channel_in_unit_interval(self.g)
                    && channel_in_unit_interval(self.b)
            }
        }
    };
}

rgb_impl!(EncodedSrgb);
rgb_impl!(LinearSrgb);
rgb_impl!(EncodedDisplayP3);
rgb_impl!(LinearDisplayP3);
rgb_impl!(EncodedAdobeRgb1998);
rgb_impl!(LinearAdobeRgb1998);
rgb_impl!(EncodedRec709);
rgb_impl!(LinearRec709);

impl EncodedSrgb {
    pub fn from_hex(hex: &str) -> Result<Self, ColorError> {
        let value = hex.trim().strip_prefix('#').unwrap_or(hex.trim());
        let expanded;
        let digits = match value.len() {
            3 => {
                expanded = expand_short_hex(value);
                expanded.as_str()
            }
            6 => value,
            found => return Err(ColorError::InvalidHexLength { found }),
        };

        let red = u8::from_str_radix(&digits[0..2], 16).map_err(|_| ColorError::InvalidHexDigit)?;
        let green =
            u8::from_str_radix(&digits[2..4], 16).map_err(|_| ColorError::InvalidHexDigit)?;
        let blue =
            u8::from_str_radix(&digits[4..6], 16).map_err(|_| ColorError::InvalidHexDigit)?;

        Self::from_rgb_u8(red, green, blue)
    }

    pub fn from_rgb_u8(r: u8, g: u8, b: u8) -> Result<Self, ColorError> {
        Self::from_rgb_u16(u16::from(r), u16::from(g), u16::from(b))
    }

    pub fn from_rgb_u16(r: u16, g: u16, b: u16) -> Result<Self, ColorError> {
        validate_rgb_u8("r", r)?;
        validate_rgb_u8("g", g)?;
        validate_rgb_u8("b", b)?;

        Ok(Self {
            r: f64::from(r) / 255.0,
            g: f64::from(g) / 255.0,
            b: f64::from(b) / 255.0,
        })
    }

    pub fn from_rgb_f64(r: f64, g: f64, b: f64) -> Result<Self, ColorError> {
        validate_unit_channel("r", r)?;
        validate_unit_channel("g", g)?;
        validate_unit_channel("b", b)?;

        Ok(Self { r, g, b })
    }

    pub fn from_hsl(h: f64, s: f64, l: f64) -> Result<Self, ColorError> {
        validate_finite("h", h)?;
        validate_unit_channel("s", s)?;
        validate_unit_channel("l", l)?;

        if s == 0.0 {
            return Ok(Self { r: l, g: l, b: l });
        }

        let hue = h.rem_euclid(360.0) / 360.0;
        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        Ok(Self {
            r: hue_to_rgb(p, q, hue + 1.0 / 3.0),
            g: hue_to_rgb(p, q, hue),
            b: hue_to_rgb(p, q, hue - 1.0 / 3.0),
        })
    }
}

impl XyzD65 {
    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

impl Oklab {
    #[must_use]
    pub const fn new(l: f64, a: f64, b: f64) -> Self {
        Self { l, a, b }
    }
}

impl Oklch {
    #[must_use]
    pub const fn new(l: f64, c: f64, h: f64) -> Self {
        Self { l, c, h }
    }
}

fn expand_short_hex(value: &str) -> String {
    let mut expanded = String::with_capacity(6);
    for ch in value.chars() {
        expanded.push(ch);
        expanded.push(ch);
    }
    expanded
}

fn validate_rgb_u8(channel: &'static str, value: u16) -> Result<(), ColorError> {
    if value <= 255 {
        Ok(())
    } else {
        Err(ColorError::InvalidRgbChannel { channel, value })
    }
}

fn validate_unit_channel(channel: &'static str, value: f64) -> Result<(), ColorError> {
    validate_finite(channel, value)?;

    if (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(ColorError::InvalidUnitInterval { channel })
    }
}

fn validate_finite(channel: &'static str, value: f64) -> Result<(), ColorError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(ColorError::NonFiniteChannel { channel })
    }
}

fn channel_in_unit_interval(value: f64) -> bool {
    value.is_finite() && (-GAMUT_EPSILON..=1.0 + GAMUT_EPSILON).contains(&value)
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}
