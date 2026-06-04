use crate::error::OciIdError;
use crate::registry::Registry;
use std::fmt;

pub const OCI_VERSION: u8 = 1;
pub const STEPS_PER_FAMILY: u16 = 360;
pub const OFFSET_PRECISION: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FamilyCode([u8; 2]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FamilyId {
    pub index: u8,
    pub code: FamilyCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StepId {
    pub anchor: u8,
    pub lightness: u8,
    pub chroma: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OklchOffset {
    pub lightness: f64,
    pub chroma: f64,
    pub hue: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OciId {
    pub version: u8,
    pub family: FamilyId,
    pub step: StepId,
    pub offset: Option<OklchOffset>,
}

impl FamilyCode {
    pub fn new(code: &str) -> Result<Self, OciIdError> {
        let bytes = code.as_bytes();
        if bytes.len() != 2 || !bytes.iter().all(u8::is_ascii_uppercase) {
            return Err(OciIdError::InvalidFamilyCode {
                found: code.to_string(),
            });
        }

        Ok(Self([bytes[0], bytes[1]]))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).expect("FamilyCode stores uppercase ASCII bytes")
    }
}

impl fmt::Display for FamilyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FamilyId {
    pub fn new(index: u8, code: &str) -> Result<Self, OciIdError> {
        Ok(Self {
            index,
            code: FamilyCode::new(code)?,
        })
    }

    pub fn parse(input: &str) -> Result<Self, OciIdError> {
        if input.len() != 4 {
            return Err(OciIdError::InvalidFamilyId {
                found: input.to_string(),
            });
        }

        let index = input[0..2]
            .parse::<u8>()
            .map_err(|_| OciIdError::InvalidFamilyId {
                found: input.to_string(),
            })?;
        let code = FamilyCode::new(&input[2..4])?;

        Ok(Self { index, code })
    }
}

impl fmt::Display for FamilyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}{}", self.index, self.code)
    }
}

impl StepId {
    pub fn new(anchor: u8, lightness: u8, chroma: u8) -> Result<Self, OciIdError> {
        validate_range("anchor", anchor, 1, 3)?;
        validate_range("lightness", lightness, 1, 12)?;
        validate_range("chroma", chroma, 1, 10)?;

        Ok(Self {
            anchor,
            lightness,
            chroma,
        })
    }

    pub fn from_step_number(step_number: u16) -> Result<Self, OciIdError> {
        if !(1..=STEPS_PER_FAMILY).contains(&step_number) {
            return Err(OciIdError::InvalidStepNumber {
                found: step_number.to_string(),
            });
        }

        let n = step_number - 1;
        let anchor = (n / 120) + 1;
        let within_anchor = n % 120;
        let lightness = (within_anchor / 10) + 1;
        let chroma = (within_anchor % 10) + 1;

        Self::new(anchor as u8, lightness as u8, chroma as u8)
    }

    #[must_use]
    pub fn step_number(self) -> u16 {
        ((u16::from(self.anchor) - 1) * 120)
            + ((u16::from(self.lightness) - 1) * 10)
            + u16::from(self.chroma)
    }
}

impl fmt::Display for StepId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "A{}-L{:02}-C{:02}",
            self.anchor, self.lightness, self.chroma
        )
    }
}

impl OklchOffset {
    #[must_use]
    pub const fn new(lightness: f64, chroma: f64, hue: f64) -> Self {
        Self {
            lightness,
            chroma,
            hue,
        }
    }

    pub fn parse(input: &str) -> Result<Self, OciIdError> {
        let parts: Vec<&str> = input.split(',').collect();
        if parts.len() != 3 {
            return Err(OciIdError::InvalidOffset {
                found: input.to_string(),
            });
        }

        Ok(Self {
            lightness: parse_offset_component(parts[0], 'L', input)?,
            chroma: parse_offset_component(parts[1], 'C', input)?,
            hue: parse_offset_component(parts[2], 'H', input)?,
        })
    }
}

impl fmt::Display for OklchOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "L{:+.6},C{:+.6},H{:+.6}",
            self.lightness, self.chroma, self.hue
        )
    }
}

impl OciId {
    #[must_use]
    pub const fn new(family: FamilyId, step: StepId, offset: Option<OklchOffset>) -> Self {
        Self {
            version: OCI_VERSION,
            family,
            step,
            offset,
        }
    }

    pub fn parse(input: &str) -> Result<Self, OciIdError> {
        let registry = Registry::load_frozen()?;
        Self::parse_with_registry(input, &registry)
    }

    pub fn parse_with_registry(input: &str, registry: &Registry) -> Result<Self, OciIdError> {
        let mut split = input.split('@');
        let base = split.next().ok_or(OciIdError::InvalidFormat)?;
        let offset = match split.next() {
            Some(raw) if !raw.is_empty() => Some(OklchOffset::parse(raw)?),
            Some(_) => {
                return Err(OciIdError::InvalidOffset {
                    found: input.to_string(),
                });
            }
            None => None,
        };
        if split.next().is_some() {
            return Err(OciIdError::InvalidFormat);
        }

        let parts: Vec<&str> = base.split('-').collect();
        if parts.len() != 4 && parts.len() != 6 {
            return Err(OciIdError::InvalidFormat);
        }
        if parts[0] != "OCI" {
            return Err(OciIdError::InvalidPrefix);
        }
        if parts[1] != "1" {
            return Err(OciIdError::InvalidVersion {
                found: parts[1].to_string(),
            });
        }

        let family = FamilyId::parse(parts[2])?;
        registry.validate_family_id(family)?;

        let step = if parts.len() == 4 {
            parse_short_step(parts[3])?
        } else {
            parse_full_step(parts[3], parts[4], parts[5])?
        };
        registry.validate_step_id(family, step)?;

        Ok(Self::new(family, step, offset))
    }

    #[must_use]
    pub fn to_short_string(&self) -> String {
        let mut out = format!(
            "OCI-{}-{}-{:03}",
            self.version,
            self.family,
            self.step.step_number()
        );
        if let Some(offset) = self.offset {
            out.push('@');
            out.push_str(&offset.to_string());
        }
        out
    }

    #[must_use]
    pub fn to_full_string(&self) -> String {
        let mut out = format!("OCI-{}-{}-{}", self.version, self.family, self.step);
        if let Some(offset) = self.offset {
            out.push('@');
            out.push_str(&offset.to_string());
        }
        out
    }
}

impl fmt::Display for OciId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_full_string())
    }
}

fn parse_short_step(input: &str) -> Result<StepId, OciIdError> {
    if input.len() != 3 || !input.as_bytes().iter().all(u8::is_ascii_digit) {
        return Err(OciIdError::InvalidStepNumber {
            found: input.to_string(),
        });
    }

    let number = input
        .parse::<u16>()
        .map_err(|_| OciIdError::InvalidStepNumber {
            found: input.to_string(),
        })?;
    StepId::from_step_number(number)
}

fn parse_full_step(anchor: &str, lightness: &str, chroma: &str) -> Result<StepId, OciIdError> {
    let anchor = parse_prefixed_u8("anchor", anchor, 'A', 1)?;
    let lightness = parse_prefixed_u8("lightness", lightness, 'L', 2)?;
    let chroma = parse_prefixed_u8("chroma", chroma, 'C', 2)?;
    StepId::new(anchor, lightness, chroma)
}

fn parse_prefixed_u8(
    component: &'static str,
    input: &str,
    prefix: char,
    digits: usize,
) -> Result<u8, OciIdError> {
    if !input.starts_with(prefix) || input.len() != 1 + digits {
        return Err(OciIdError::InvalidStepComponent {
            component,
            found: input.to_string(),
        });
    }
    let value = input[1..]
        .parse::<u8>()
        .map_err(|_| OciIdError::InvalidStepComponent {
            component,
            found: input.to_string(),
        })?;
    Ok(value)
}

fn validate_range(component: &'static str, value: u8, min: u8, max: u8) -> Result<(), OciIdError> {
    if (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(OciIdError::InvalidStepComponent {
            component,
            found: value.to_string(),
        })
    }
}

fn parse_offset_component(input: &str, label: char, full: &str) -> Result<f64, OciIdError> {
    let bytes = input.as_bytes();
    if bytes.len() < 10 || bytes[0] != label as u8 || (bytes[1] != b'+' && bytes[1] != b'-') {
        return Err(OciIdError::InvalidOffset {
            found: full.to_string(),
        });
    }

    let numeric = &input[2..];
    let Some(dot) = numeric.find('.') else {
        return Err(OciIdError::InvalidOffset {
            found: full.to_string(),
        });
    };
    let numeric_bytes = numeric.as_bytes();
    if numeric.len() - dot - 1 != OFFSET_PRECISION
        || !numeric_bytes[..dot].iter().all(u8::is_ascii_digit)
        || !numeric_bytes[dot + 1..].iter().all(u8::is_ascii_digit)
    {
        return Err(OciIdError::InvalidOffset {
            found: full.to_string(),
        });
    }

    let magnitude = numeric
        .parse::<f64>()
        .map_err(|_| OciIdError::InvalidOffset {
            found: full.to_string(),
        })?;
    if !magnitude.is_finite() {
        return Err(OciIdError::InvalidOffset {
            found: full.to_string(),
        });
    }

    Ok(if bytes[1] == b'-' {
        -magnitude
    } else {
        magnitude
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::OciIdError;

    #[test]
    fn maps_short_id_to_full_id() {
        let id = OciId::parse("OCI-1-46PK-236").unwrap();
        assert_eq!(id.to_full_string(), "OCI-1-46PK-A2-L12-C06");
    }

    #[test]
    fn maps_full_id_back_to_short_id() {
        let id = OciId::parse("OCI-1-46PK-A2-L12-C06").unwrap();
        assert_eq!(id.to_short_string(), "OCI-1-46PK-236");
    }

    #[test]
    fn parses_and_formats_offsets() {
        let id = OciId::parse("OCI-1-46PK-236@L+0.002134,C-0.001042,H+0.218400").unwrap();
        assert_eq!(
            id.to_full_string(),
            "OCI-1-46PK-A2-L12-C06@L+0.002134,C-0.001042,H+0.218400"
        );
        assert_eq!(
            id.to_short_string(),
            "OCI-1-46PK-236@L+0.002134,C-0.001042,H+0.218400"
        );
    }

    #[test]
    fn rejects_invalid_family_code() {
        assert!(OciId::parse("OCI-1-46X!-236").is_err());
        assert!(OciId::parse("OCI-1-99PK-236").is_err());
    }

    #[test]
    fn rejects_family_index_code_mismatch() {
        let error = OciId::parse("OCI-1-46RD-236").unwrap_err();
        assert!(matches!(error, OciIdError::FamilyIndexCodeMismatch { .. }));
    }

    #[test]
    fn rejects_invalid_step_number() {
        assert!(OciId::parse("OCI-1-46PK-000").is_err());
        assert!(OciId::parse("OCI-1-46PK-361").is_err());
    }

    #[test]
    fn rejects_invalid_anchor_lightness_or_chroma() {
        assert!(OciId::parse("OCI-1-46PK-A4-L12-C06").is_err());
        assert!(OciId::parse("OCI-1-46PK-A2-L13-C06").is_err());
        assert!(OciId::parse("OCI-1-46PK-A2-L12-C11").is_err());
    }
}
