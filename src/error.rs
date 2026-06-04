use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorError {
    InvalidHexLength { found: usize },
    InvalidHexDigit,
    InvalidRgbChannel { channel: &'static str, value: u16 },
    NonFiniteChannel { channel: &'static str },
    InvalidUnitInterval { channel: &'static str },
}

impl fmt::Display for ColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHexLength { found } => {
                write!(
                    f,
                    "invalid HEX length: expected 3 or 6 digits, found {found}"
                )
            }
            Self::InvalidHexDigit => write!(f, "invalid HEX digit"),
            Self::InvalidRgbChannel { channel, value } => {
                write!(
                    f,
                    "invalid RGB {channel} channel: {value} is outside 0..=255"
                )
            }
            Self::NonFiniteChannel { channel } => {
                write!(f, "invalid {channel} channel: value must be finite")
            }
            Self::InvalidUnitInterval { channel } => {
                write!(f, "invalid {channel} channel: value must be in 0.0..=1.0")
            }
        }
    }
}

impl Error for ColorError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OciIdError {
    InvalidFormat,
    InvalidPrefix,
    InvalidVersion {
        found: String,
    },
    InvalidFamilyId {
        found: String,
    },
    InvalidFamilyCode {
        found: String,
    },
    UnknownFamily {
        found: String,
    },
    FamilyIndexCodeMismatch {
        found: String,
        expected: String,
    },
    InvalidStepNumber {
        found: String,
    },
    InvalidStepComponent {
        component: &'static str,
        found: String,
    },
    InvalidOffset {
        found: String,
    },
    Registry(RegistryError),
}

impl fmt::Display for OciIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => write!(f, "invalid OCI ID format"),
            Self::InvalidPrefix => write!(f, "invalid OCI ID prefix: expected OCI"),
            Self::InvalidVersion { found } => {
                write!(f, "invalid OCI ID version: expected 1, found {found}")
            }
            Self::InvalidFamilyId { found } => {
                write!(f, "invalid OCI family id: expected NNCODE, found {found}")
            }
            Self::InvalidFamilyCode { found } => {
                write!(
                    f,
                    "invalid OCI family code: expected two uppercase ASCII letters, found {found}"
                )
            }
            Self::UnknownFamily { found } => write!(f, "unknown OCI family: {found}"),
            Self::FamilyIndexCodeMismatch { found, expected } => {
                write!(
                    f,
                    "OCI family index/code mismatch: found {found}, expected {expected}"
                )
            }
            Self::InvalidStepNumber { found } => {
                write!(f, "invalid OCI short step number: {found}")
            }
            Self::InvalidStepComponent { component, found } => {
                write!(f, "invalid OCI {component} component: {found}")
            }
            Self::InvalidOffset { found } => write!(f, "invalid OCI OKLCH offset: {found}"),
            Self::Registry(error) => write!(f, "registry error while parsing OCI ID: {error}"),
        }
    }
}

impl Error for OciIdError {}

impl From<RegistryError> for OciIdError {
    fn from(value: RegistryError) -> Self {
        Self::Registry(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    FrozenJsonParse {
        file: &'static str,
        line: usize,
        reason: String,
    },
    InvalidFamilyCount {
        found: usize,
    },
    InvalidStepCount {
        found: usize,
    },
    InvalidStepsPerFamily {
        family: String,
        found: usize,
    },
    MissingFamily {
        family: String,
    },
    DuplicateFamily {
        family: String,
    },
    DuplicateStep {
        id: String,
    },
    InvalidStepMapping {
        id: String,
        reason: String,
    },
    ChecksumMismatch {
        path: &'static str,
        expected: String,
        actual: String,
    },
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FrozenJsonParse { file, line, reason } => {
                write!(
                    f,
                    "failed to parse frozen JSON {file} at line {line}: {reason}"
                )
            }
            Self::InvalidFamilyCount { found } => {
                write!(
                    f,
                    "invalid registry family count: expected 64, found {found}"
                )
            }
            Self::InvalidStepCount { found } => {
                write!(
                    f,
                    "invalid registry step count: expected 23040, found {found}"
                )
            }
            Self::InvalidStepsPerFamily { family, found } => {
                write!(
                    f,
                    "invalid registry step count for {family}: expected 360, found {found}"
                )
            }
            Self::MissingFamily { family } => write!(f, "missing registry family {family}"),
            Self::DuplicateFamily { family } => write!(f, "duplicate registry family {family}"),
            Self::DuplicateStep { id } => write!(f, "duplicate registry step {id}"),
            Self::InvalidStepMapping { id, reason } => {
                write!(f, "invalid registry step mapping for {id}: {reason}")
            }
            Self::ChecksumMismatch {
                path,
                expected,
                actual,
            } => write!(
                f,
                "registry checksum mismatch for {path}: expected {expected}, got {actual}"
            ),
        }
    }
}

impl Error for RegistryError {}

#[derive(Debug, Clone, PartialEq)]
pub enum OciPipelineError {
    Color(ColorError),
    Id(OciIdError),
    Registry(RegistryError),
    InvalidColorComponent { component: &'static str, value: f64 },
    MissingBaseStep { id: String },
    NoNearestStep,
}

impl fmt::Display for OciPipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Color(error) => write!(f, "color input error: {error}"),
            Self::Id(error) => write!(f, "OCI ID error: {error}"),
            Self::Registry(error) => write!(f, "registry error: {error}"),
            Self::InvalidColorComponent { component, value } => {
                write!(f, "invalid {component} component: {value} must be finite")
            }
            Self::MissingBaseStep { id } => write!(f, "missing registered base step {id}"),
            Self::NoNearestStep => write!(f, "no nearest registered OCI step found"),
        }
    }
}

impl Error for OciPipelineError {}

impl From<ColorError> for OciPipelineError {
    fn from(value: ColorError) -> Self {
        Self::Color(value)
    }
}

impl From<OciIdError> for OciPipelineError {
    fn from(value: OciIdError) -> Self {
        Self::Id(value)
    }
}

impl From<RegistryError> for OciPipelineError {
    fn from(value: RegistryError) -> Self {
        Self::Registry(value)
    }
}
