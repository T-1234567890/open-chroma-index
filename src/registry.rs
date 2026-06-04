use crate::error::{OciIdError, RegistryError};
use crate::id::{FamilyId, StepId};
use std::collections::HashSet;

pub const FAMILY_COUNT: usize = 64;
pub const STEPS_PER_FAMILY_COUNT: usize = 360;
pub const TOTAL_REGISTERED_BASE_STEPS: usize = 23_040;

const FAMILIES_JSON: &str = include_str!("../registry/v1/families.json");
const STEPS_JSON: &str = include_str!("../registry/v1/steps.json");
const TEST_VECTORS_JSON: &str = include_str!("../registry/v1/test-vectors.json");
const METADATA_JSON: &str = include_str!("../registry/v1/metadata.json");
const CHECKSUMS_JSON: &str = include_str!("../registry/v1/checksums.json");
const SCHEMA_JSON: &str = include_str!("../registry/v1/schema.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrySource {
    FrozenJson,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Family {
    pub id: FamilyId,
    pub name: String,
    pub group: String,
    pub hue_start: f64,
    pub hue_end: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryStep {
    pub id: String,
    pub short_id: String,
    pub family_id: FamilyId,
    pub step_number: u16,
    pub step: StepId,
    pub hue: f64,
    pub lightness: f64,
    pub chroma_ratio: f64,
    pub chroma: f64,
}

#[derive(Debug, Clone)]
pub struct Registry {
    families: Vec<Family>,
    steps: Vec<RegistryStep>,
    source: RegistrySource,
}

impl Registry {
    pub fn load_frozen() -> Result<Self, RegistryError> {
        validate_frozen_checksums()?;

        let families = parse_families(FAMILIES_JSON)?;
        let steps = parse_steps(STEPS_JSON)?;
        let registry = Self {
            families,
            steps,
            source: RegistrySource::FrozenJson,
        };
        registry.validate()?;
        Ok(registry)
    }

    #[must_use]
    pub fn source(&self) -> RegistrySource {
        self.source
    }

    #[must_use]
    pub fn families(&self) -> &[Family] {
        &self.families
    }

    #[must_use]
    pub fn steps(&self) -> &[RegistryStep] {
        &self.steps
    }

    #[must_use]
    pub fn frozen_families_json() -> &'static str {
        FAMILIES_JSON
    }

    #[must_use]
    pub fn frozen_steps_json() -> &'static str {
        STEPS_JSON
    }

    #[must_use]
    pub fn frozen_test_vectors_json() -> &'static str {
        TEST_VECTORS_JSON
    }

    #[must_use]
    pub fn frozen_metadata_json() -> &'static str {
        METADATA_JSON
    }

    #[must_use]
    pub fn frozen_checksums_json() -> &'static str {
        CHECKSUMS_JSON
    }

    #[must_use]
    pub fn frozen_schema_json() -> &'static str {
        SCHEMA_JSON
    }

    pub fn validate(&self) -> Result<(), RegistryError> {
        if self.families.len() != FAMILY_COUNT {
            return Err(RegistryError::InvalidFamilyCount {
                found: self.families.len(),
            });
        }
        if self.steps.len() != TOTAL_REGISTERED_BASE_STEPS {
            return Err(RegistryError::InvalidStepCount {
                found: self.steps.len(),
            });
        }

        let mut seen_families = HashSet::new();
        let mut family_counts = [0usize; FAMILY_COUNT];
        for family in &self.families {
            if !seen_families.insert(family.id) {
                return Err(RegistryError::DuplicateFamily {
                    family: family.id.to_string(),
                });
            }
            if usize::from(family.id.index) >= FAMILY_COUNT {
                return Err(RegistryError::MissingFamily {
                    family: family.id.to_string(),
                });
            }
        }

        let mut seen_steps = HashSet::new();
        for step in &self.steps {
            self.validate_family_id(step.family_id).map_err(|error| {
                RegistryError::InvalidStepMapping {
                    id: step.id.clone(),
                    reason: error.to_string(),
                }
            })?;

            if !seen_steps.insert(step.id.clone()) || !seen_steps.insert(step.short_id.clone()) {
                return Err(RegistryError::DuplicateStep {
                    id: step.id.clone(),
                });
            }

            let expected_step_number = step.step.step_number();
            if step.step_number != expected_step_number {
                return Err(RegistryError::InvalidStepMapping {
                    id: step.id.clone(),
                    reason: format!(
                        "step number {} does not match {}",
                        step.step_number, expected_step_number
                    ),
                });
            }

            let expected_full_id = format!("OCI-1-{}-{}", step.family_id, step.step);
            let expected_short_id = format!("OCI-1-{}-{:03}", step.family_id, step.step_number);
            if step.id != expected_full_id || step.short_id != expected_short_id {
                return Err(RegistryError::InvalidStepMapping {
                    id: step.id.clone(),
                    reason: format!("expected {expected_full_id}/{expected_short_id}"),
                });
            }

            family_counts[usize::from(step.family_id.index)] += 1;
        }

        for family in &self.families {
            let found = family_counts[usize::from(family.id.index)];
            if found != STEPS_PER_FAMILY_COUNT {
                return Err(RegistryError::InvalidStepsPerFamily {
                    family: family.id.to_string(),
                    found,
                });
            }
        }

        Ok(())
    }

    pub fn validate_family_id(&self, family_id: FamilyId) -> Result<(), OciIdError> {
        let Some(family) = self
            .families
            .iter()
            .find(|family| family.id.index == family_id.index)
        else {
            return Err(OciIdError::UnknownFamily {
                found: family_id.to_string(),
            });
        };

        if family.id != family_id {
            return Err(OciIdError::FamilyIndexCodeMismatch {
                found: family_id.to_string(),
                expected: family.id.to_string(),
            });
        }

        Ok(())
    }

    pub fn validate_step_id(&self, family_id: FamilyId, step: StepId) -> Result<(), OciIdError> {
        if self.find_step(family_id, step).is_some() {
            Ok(())
        } else {
            Err(OciIdError::InvalidStepNumber {
                found: step.step_number().to_string(),
            })
        }
    }

    #[must_use]
    pub fn find_family(&self, family_id: FamilyId) -> Option<&Family> {
        self.families.iter().find(|family| family.id == family_id)
    }

    #[must_use]
    pub fn find_step(&self, family_id: FamilyId, step: StepId) -> Option<&RegistryStep> {
        self.steps
            .iter()
            .find(|record| record.family_id == family_id && record.step == step)
    }
}

fn parse_families(json: &'static str) -> Result<Vec<Family>, RegistryError> {
    json_object_lines(json)
        .map(|(line, object)| parse_family(line, object))
        .collect()
}

fn parse_steps(json: &'static str) -> Result<Vec<RegistryStep>, RegistryError> {
    json_object_lines(json)
        .map(|(line, object)| parse_step(line, object))
        .collect()
}

fn parse_family(line: usize, object: &str) -> Result<Family, RegistryError> {
    let id = json_string("families.json", line, object, "id")?;
    let index = json_u8("families.json", line, object, "index")?;
    let code = json_string("families.json", line, object, "code")?;
    let family_id =
        FamilyId::new(index, &code).map_err(|error| RegistryError::FrozenJsonParse {
            file: "families.json",
            line,
            reason: error.to_string(),
        })?;

    if id != family_id.to_string() {
        return Err(RegistryError::FrozenJsonParse {
            file: "families.json",
            line,
            reason: format!("id {id} does not match index/code {family_id}"),
        });
    }

    Ok(Family {
        id: family_id,
        name: json_string("families.json", line, object, "name")?,
        group: json_string("families.json", line, object, "group")?,
        hue_start: json_f64("families.json", line, object, "hueStart")?,
        hue_end: json_f64("families.json", line, object, "hueEnd")?,
    })
}

fn parse_step(line: usize, object: &str) -> Result<RegistryStep, RegistryError> {
    let family_index = json_u8("steps.json", line, object, "familyIndex")?;
    let family_code = json_string("steps.json", line, object, "familyCode")?;
    let family_id = FamilyId::new(family_index, &family_code).map_err(|error| {
        RegistryError::FrozenJsonParse {
            file: "steps.json",
            line,
            reason: error.to_string(),
        }
    })?;
    let family_id_string = json_string("steps.json", line, object, "familyId")?;
    if family_id.to_string() != family_id_string {
        return Err(RegistryError::FrozenJsonParse {
            file: "steps.json",
            line,
            reason: format!("familyId {family_id_string} does not match {family_id}"),
        });
    }

    let step_number = json_u16("steps.json", line, object, "stepNumber")?;
    let anchor = json_u8("steps.json", line, object, "anchor")?;
    let lightness_level = json_u8("steps.json", line, object, "lightnessLevel")?;
    let chroma_level = json_u8("steps.json", line, object, "chromaLevel")?;
    let step = StepId::new(anchor, lightness_level, chroma_level).map_err(|error| {
        RegistryError::FrozenJsonParse {
            file: "steps.json",
            line,
            reason: error.to_string(),
        }
    })?;
    if step.step_number() != step_number {
        return Err(RegistryError::FrozenJsonParse {
            file: "steps.json",
            line,
            reason: format!(
                "stepNumber {step_number} does not match anchor/lightness/chroma {}",
                step.step_number()
            ),
        });
    }

    Ok(RegistryStep {
        id: json_string("steps.json", line, object, "id")?,
        short_id: json_string("steps.json", line, object, "shortId")?,
        family_id,
        step_number,
        step,
        hue: json_f64("steps.json", line, object, "hue")?,
        lightness: json_f64("steps.json", line, object, "lightness")?,
        chroma_ratio: json_f64("steps.json", line, object, "chromaRatio")?,
        chroma: json_f64("steps.json", line, object, "chroma")?,
    })
}

fn json_object_lines(json: &'static str) -> impl Iterator<Item = (usize, &'static str)> {
    json.lines().enumerate().filter_map(|(index, line)| {
        let trimmed = line.trim().trim_end_matches(',');
        trimmed.starts_with('{').then_some((index + 1, trimmed))
    })
}

fn json_string(
    file: &'static str,
    line: usize,
    object: &str,
    key: &str,
) -> Result<String, RegistryError> {
    let rest = json_value_rest(file, line, object, key)?.trim_start();
    if !rest.starts_with('"') {
        return Err(json_error(file, line, key, "expected string value"));
    }
    let value = &rest[1..];
    let Some(end) = value.find('"') else {
        return Err(json_error(file, line, key, "unterminated string value"));
    };
    Ok(value[..end].to_string())
}

fn json_u8(file: &'static str, line: usize, object: &str, key: &str) -> Result<u8, RegistryError> {
    let value = json_number(file, line, object, key)?;
    value
        .parse::<u8>()
        .map_err(|_| json_error(file, line, key, "expected u8 value"))
}

fn json_u16(
    file: &'static str,
    line: usize,
    object: &str,
    key: &str,
) -> Result<u16, RegistryError> {
    let value = json_number(file, line, object, key)?;
    value
        .parse::<u16>()
        .map_err(|_| json_error(file, line, key, "expected u16 value"))
}

fn json_f64(
    file: &'static str,
    line: usize,
    object: &str,
    key: &str,
) -> Result<f64, RegistryError> {
    let value = json_number(file, line, object, key)?;
    let parsed = value
        .parse::<f64>()
        .map_err(|_| json_error(file, line, key, "expected f64 value"))?;
    if parsed.is_finite() {
        Ok(parsed)
    } else {
        Err(json_error(file, line, key, "expected finite f64 value"))
    }
}

fn json_number<'a>(
    file: &'static str,
    line: usize,
    object: &'a str,
    key: &str,
) -> Result<&'a str, RegistryError> {
    let rest = json_value_rest(file, line, object, key)?.trim_start();
    let end = rest
        .find([',', '}'])
        .ok_or_else(|| json_error(file, line, key, "unterminated number value"))?;
    Ok(rest[..end].trim())
}

fn json_value_rest<'a>(
    file: &'static str,
    line: usize,
    object: &'a str,
    key: &str,
) -> Result<&'a str, RegistryError> {
    let marker = format!("\"{key}\":");
    if let Some(start) = object.find(&marker) {
        return Ok(&object[start + marker.len()..]);
    }

    let spaced_marker = format!("\"{key}\": ");
    let Some(start) = object.find(&spaced_marker) else {
        return Err(json_error(file, line, key, "missing key"));
    };
    Ok(&object[start + spaced_marker.len()..])
}

fn json_error(file: &'static str, line: usize, key: &str, reason: &str) -> RegistryError {
    RegistryError::FrozenJsonParse {
        file,
        line,
        reason: format!("{key}: {reason}"),
    }
}

fn validate_frozen_checksums() -> Result<(), RegistryError> {
    validate_checksum("registry/v1/families.json", FAMILIES_JSON)?;
    validate_checksum("registry/v1/steps.json", STEPS_JSON)?;
    validate_checksum("registry/v1/test-vectors.json", TEST_VECTORS_JSON)?;
    validate_checksum("registry/v1/metadata.json", METADATA_JSON)?;
    validate_checksum("registry/v1/schema.json", SCHEMA_JSON)?;
    Ok(())
}

fn validate_checksum(path: &'static str, content: &str) -> Result<(), RegistryError> {
    let Some(expected) = checksum_for_path(path) else {
        return Err(RegistryError::FrozenJsonParse {
            file: "checksums.json",
            line: 0,
            reason: format!("missing checksum for {path}"),
        });
    };
    let actual = sha256_normalized_text_hex(content);
    if expected == actual {
        Ok(())
    } else {
        Err(RegistryError::ChecksumMismatch {
            path,
            expected,
            actual,
        })
    }
}

fn checksum_for_path(path: &str) -> Option<String> {
    for line in CHECKSUMS_JSON.lines() {
        if line.contains(path) {
            return json_string(
                "checksums.json",
                0,
                line.trim().trim_end_matches(','),
                "sha256",
            )
            .ok();
        }
    }
    None
}

#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = sha256(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[must_use]
pub fn sha256_normalized_text_hex(content: &str) -> String {
    if content.as_bytes().contains(&b'\r') {
        sha256_hex(content.replace("\r\n", "\n").replace('\r', "\n").as_bytes())
    } else {
        sha256_hex(content.as_bytes())
    }
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a_2f98,
        0x7137_4491,
        0xb5c0_fbcf,
        0xe9b5_dba5,
        0x3956_c25b,
        0x59f1_11f1,
        0x923f_82a4,
        0xab1c_5ed5,
        0xd807_aa98,
        0x1283_5b01,
        0x2431_85be,
        0x550c_7dc3,
        0x72be_5d74,
        0x80de_b1fe,
        0x9bdc_06a7,
        0xc19b_f174,
        0xe49b_69c1,
        0xefbe_4786,
        0x0fc1_9dc6,
        0x240c_a1cc,
        0x2de9_2c6f,
        0x4a74_84aa,
        0x5cb0_a9dc,
        0x76f9_88da,
        0x983e_5152,
        0xa831_c66d,
        0xb003_27c8,
        0xbf59_7fc7,
        0xc6e0_0bf3,
        0xd5a7_9147,
        0x06ca_6351,
        0x1429_2967,
        0x27b7_0a85,
        0x2e1b_2138,
        0x4d2c_6dfc,
        0x5338_0d13,
        0x650a_7354,
        0x766a_0abb,
        0x81c2_c92e,
        0x9272_2c85,
        0xa2bf_e8a1,
        0xa81a_664b,
        0xc24b_8b70,
        0xc76c_51a3,
        0xd192_e819,
        0xd699_0624,
        0xf40e_3585,
        0x106a_a070,
        0x19a4_c116,
        0x1e37_6c08,
        0x2748_774c,
        0x34b0_bcb5,
        0x391c_0cb3,
        0x4ed8_aa4a,
        0x5b9c_ca4f,
        0x682e_6ff3,
        0x748f_82ee,
        0x78a5_636f,
        0x84c8_7814,
        0x8cc7_0208,
        0x90be_fffa,
        0xa450_6ceb,
        0xbef9_a3f7,
        0xc671_78f2,
    ];

    let mut h: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];

    let bit_len = (bytes.len() as u64) * 8;
    let mut message = bytes.to_vec();
    message.push(0x80);
    while (message.len() % 64) != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in message.chunks(64) {
        let mut w = [0u32; 64];
        for (i, word) in w.iter_mut().take(16).enumerate() {
            let start = i * 4;
            *word = u32::from_be_bytes([
                chunk[start],
                chunk[start + 1],
                chunk[start + 2],
                chunk[start + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_families_json_has_exactly_64_families() {
        let registry = Registry::load_frozen().unwrap();
        assert_eq!(registry.families().len(), FAMILY_COUNT);
    }

    #[test]
    fn frozen_steps_json_has_exactly_23040_steps() {
        let registry = Registry::load_frozen().unwrap();
        assert_eq!(registry.steps().len(), TOTAL_REGISTERED_BASE_STEPS);
    }

    #[test]
    fn each_family_has_exactly_360_steps() {
        let registry = Registry::load_frozen().unwrap();
        for family in registry.families() {
            let count = registry
                .steps()
                .iter()
                .filter(|step| step.family_id == family.id)
                .count();
            assert_eq!(count, STEPS_PER_FAMILY_COUNT);
        }
    }

    #[test]
    fn family_46pk_exists() {
        let registry = Registry::load_frozen().unwrap();
        let family = registry
            .families()
            .iter()
            .find(|family| family.id.to_string() == "46PK")
            .unwrap();
        assert_eq!(family.id.index, 46);
        assert_eq!(family.id.code.to_string(), "PK");
    }

    #[test]
    fn runtime_loader_reads_frozen_json() {
        let registry = Registry::load_frozen().unwrap();
        assert_eq!(registry.source(), RegistrySource::FrozenJson);
        assert!(Registry::frozen_steps_json().contains("\"shortId\":\"OCI-1-46PK-236\""));
        assert!(
            registry
                .steps()
                .iter()
                .any(|step| step.short_id == "OCI-1-46PK-236")
        );
    }

    #[test]
    fn checksum_validation_is_stable_across_crlf_checkouts() {
        let lf = Registry::frozen_families_json();
        let crlf = lf.replace('\n', "\r\n");
        assert_eq!(
            sha256_normalized_text_hex(lf),
            sha256_normalized_text_hex(&crlf)
        );
    }
}
