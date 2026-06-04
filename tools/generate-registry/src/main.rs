use std::fs;
use std::io;
use std::path::Path;

const LIGHTNESS_LEVELS: [f64; 12] = [
    0.06, 0.14, 0.22, 0.30, 0.38, 0.46, 0.54, 0.62, 0.70, 0.78, 0.86, 0.94,
];
const CHROMA_RATIOS: [f64; 10] = [0.05, 0.12, 0.22, 0.34, 0.46, 0.58, 0.70, 0.82, 0.92, 1.00];

const FAMILIES: [(u8, &str, &str, &str); 64] = [
    (0, "RD", "Red", "Reds to Yellows"),
    (1, "VR", "Vermilion Red", "Reds to Yellows"),
    (2, "VM", "Vermilion", "Reds to Yellows"),
    (3, "CR", "Coral Red", "Reds to Yellows"),
    (4, "CO", "Coral", "Reds to Yellows"),
    (5, "PO", "Peach Orange", "Reds to Yellows"),
    (6, "OR", "Orange", "Reds to Yellows"),
    (7, "AO", "Amber Orange", "Reds to Yellows"),
    (8, "AM", "Amber", "Reds to Yellows"),
    (9, "GD", "Gold", "Reds to Yellows"),
    (10, "YW", "Yellow", "Reds to Yellows"),
    (11, "LY", "Lemon Yellow", "Reds to Yellows"),
    (12, "LM", "Lime", "Lime to Teals"),
    (13, "LG", "Lime Green", "Lime to Teals"),
    (14, "YG", "Yellow Green", "Lime to Teals"),
    (15, "GR", "Green", "Lime to Teals"),
    (16, "FG", "Forest Green", "Lime to Teals"),
    (17, "EG", "Emerald Green", "Lime to Teals"),
    (18, "EM", "Emerald", "Lime to Teals"),
    (19, "MN", "Mint Green", "Lime to Teals"),
    (20, "MT", "Mint", "Lime to Teals"),
    (21, "SE", "Sea Green", "Lime to Teals"),
    (22, "TL", "Teal", "Lime to Teals"),
    (23, "CT", "Cyan Teal", "Lime to Teals"),
    (24, "CY", "Cyan", "Cyans to Purples"),
    (25, "AQ", "Aqua", "Cyans to Purples"),
    (26, "AZ", "Azure", "Cyans to Purples"),
    (27, "SB", "Sky Blue", "Cyans to Purples"),
    (28, "SK", "Sky", "Cyans to Purples"),
    (29, "LB", "Light Blue", "Cyans to Purples"),
    (30, "BL", "Blue", "Cyans to Purples"),
    (31, "RB", "Royal Blue", "Cyans to Purples"),
    (32, "CB", "Cobalt Blue", "Cyans to Purples"),
    (33, "NV", "Navy", "Cyans to Purples"),
    (34, "IB", "Indigo Blue", "Cyans to Purples"),
    (35, "IN", "Indigo", "Cyans to Purples"),
    (36, "IV", "Indigo Violet", "Cyans to Purples"),
    (37, "VT", "Violet", "Cyans to Purples"),
    (38, "LV", "Lavender Violet", "Lavenders to Pinks"),
    (39, "LA", "Lavender", "Lavenders to Pinks"),
    (40, "PR", "Purple", "Lavenders to Pinks"),
    (41, "BP", "Blue Purple", "Lavenders to Pinks"),
    (42, "MA", "Magenta", "Lavenders to Pinks"),
    (43, "FM", "Fuchsia Magenta", "Lavenders to Pinks"),
    (44, "FS", "Fuchsia", "Lavenders to Pinks"),
    (45, "HP", "Hot Pink", "Lavenders to Pinks"),
    (46, "PK", "Pink", "Lavenders to Pinks"),
    (47, "RP", "Rose Pink", "Lavenders to Pinks"),
    (48, "RS", "Rose", "Lavenders to Pinks"),
    (49, "WR", "Wine Red", "Lavenders to Pinks"),
    (50, "MR", "Maroon Red", "Lavenders to Pinks"),
    (51, "MU", "Maroon", "Lavenders to Pinks"),
    (52, "BR", "Brown", "Earth & Muted Tones"),
    (53, "CP", "Copper Brown", "Earth & Muted Tones"),
    (54, "TN", "Tan", "Earth & Muted Tones"),
    (55, "BG", "Beige", "Earth & Muted Tones"),
    (56, "OL", "Olive", "Earth & Muted Tones"),
    (57, "OG", "Olive Green", "Earth & Muted Tones"),
    (58, "SL", "Slate", "Grayscale & Neutrals"),
    (59, "SG", "Slate Gray", "Grayscale & Neutrals"),
    (60, "GY", "Gray", "Grayscale & Neutrals"),
    (61, "NG", "Neutral Gray", "Grayscale & Neutrals"),
    (62, "WH", "White Neutral", "Grayscale & Neutrals"),
    (63, "BK", "Black Neutral", "Grayscale & Neutrals"),
];

fn main() -> io::Result<()> {
    let registry_dir = Path::new("registry/v1");
    fs::create_dir_all(registry_dir)?;

    let families = families_json();
    let steps = steps_json();
    let test_vectors = test_vectors_json();
    let metadata = metadata_json();
    let schema = schema_json();

    fs::write(registry_dir.join("families.json"), &families)?;
    fs::write(registry_dir.join("steps.json"), &steps)?;
    fs::write(registry_dir.join("test-vectors.json"), &test_vectors)?;
    fs::write(registry_dir.join("metadata.json"), &metadata)?;
    fs::write(registry_dir.join("schema.json"), &schema)?;
    fs::write(
        registry_dir.join("checksums.json"),
        checksums_json(&families, &steps, &test_vectors, &metadata, &schema),
    )?;

    Ok(())
}

fn families_json() -> String {
    let mut out = String::from("[\n");

    for (position, (index, code, name, group)) in FAMILIES.iter().enumerate() {
        let (classification, hue_start, hue_end) = family_model(*index).families_json_fields();
        out.push_str(&format!(
            "  {{\"id\":\"{index:02}{code}\",\"index\":{index},\"code\":\"{code}\",\"name\":\"{name}\",\"group\":\"{group}\",\"classification\":\"{classification}\",\"hueStart\":{hue_start:.6},\"hueEnd\":{hue_end:.6}}}"
        ));
        if position + 1 != FAMILIES.len() {
            out.push(',');
        }
        out.push('\n');
    }

    out.push_str("]\n");
    out
}

fn steps_json() -> String {
    let mut out = String::from("[\n");
    let mut written = 0usize;
    let total = FAMILIES.len() * 3 * LIGHTNESS_LEVELS.len() * CHROMA_RATIOS.len();

    for (family_index, code, _, _) in FAMILIES {
        let model = family_model(family_index);
        for anchor in 1u8..=3 {
            let hue = model.anchor_hue(anchor);
            for (lightness_index, lightness) in LIGHTNESS_LEVELS.iter().enumerate() {
                let lightness_level = lightness_index + 1;
                for (chroma_index, chroma_ratio) in CHROMA_RATIOS.iter().enumerate() {
                    let chroma_level = chroma_index + 1;
                    let step_number = ((usize::from(anchor) - 1) * 120)
                        + ((lightness_level - 1) * 10)
                        + chroma_level;
                    let family_id = format!("{family_index:02}{code}");
                    let full_id = format!(
                        "OCI-1-{family_id}-A{anchor}-L{lightness_level:02}-C{chroma_level:02}"
                    );
                    let short_id = format!("OCI-1-{family_id}-{step_number:03}");
                    let max_chroma = model.max_chroma(*lightness);
                    let chroma = max_chroma * chroma_ratio;

                    out.push_str(&format!(
                        "  {{\"id\":\"{full_id}\",\"shortId\":\"{short_id}\",\"familyId\":\"{family_id}\",\"familyIndex\":{family_index},\"familyCode\":\"{code}\",\"stepNumber\":{step_number},\"anchor\":{anchor},\"lightnessLevel\":{lightness_level},\"chromaLevel\":{chroma_level},\"hue\":{hue:.6},\"lightness\":{lightness:.6},\"chromaRatio\":{chroma_ratio:.6},\"chroma\":{chroma:.6}}}"
                    ));

                    written += 1;
                    if written != total {
                        out.push(',');
                    }
                    out.push('\n');
                }
            }
        }
    }

    out.push_str("]\n");
    out
}

fn metadata_json() -> String {
    let max_chroma_note = "v1-beta uses deterministic semantic placeholder maxChroma rules by family class. Chromatic families use higher chroma, earth/muted families use reduced chroma, and grayscale/neutral families use very low chroma. It is not an ICC or sRGB gamut search and is frozen for registry stability.";
    format!(
        concat!(
            "{{\n",
            "  \"name\":\"Open Chroma Index Registry\",\n",
            "  \"version\":\"1-beta\",\n",
            "  \"canonicalModel\":\"OKLCH/OKLab\",\n",
            "  \"familyCount\":64,\n",
            "  \"stepsPerFamily\":360,\n",
            "  \"hueAnchorsPerFamily\":3,\n",
            "  \"lightnessLevels\":12,\n",
            "  \"chromaLevels\":10,\n",
            "  \"totalRegisteredBaseSteps\":23040,\n",
            "  \"offsetPrecision\":6,\n",
            "  \"familyModel\":\"semantic-v1-beta\",\n",
            "  \"testVectorCount\":17,\n",
            "  \"maxChromaStatus\":\"placeholder\",\n",
            "  \"maxChromaNote\":\"{}\"\n",
            "}}\n"
        ),
        json_escape(max_chroma_note)
    )
}

fn schema_json() -> String {
    include_str!("../../../registry/v1/schema.json").to_string()
}

fn test_vectors_json() -> String {
    include_str!("../../../registry/v1/test-vectors.json").to_string()
}

fn checksums_json(
    families: &str,
    steps: &str,
    test_vectors: &str,
    metadata: &str,
    schema: &str,
) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"algorithm\": \"sha256\",\n",
            "  \"files\":[\n",
            "    {{\"path\": \"registry/v1/families.json\", \"sha256\": \"{families}\"}},\n",
            "    {{\"path\": \"registry/v1/steps.json\", \"sha256\": \"{steps}\"}},\n",
            "    {{\"path\": \"registry/v1/test-vectors.json\", \"sha256\": \"{test_vectors}\"}},\n",
            "    {{\"path\": \"registry/v1/schema.json\", \"sha256\": \"{schema}\"}},\n",
            "    {{\"path\": \"registry/v1/metadata.json\", \"sha256\": \"{metadata}\"}}\n",
            "  ]\n",
            "}}\n"
        ),
        families = oci_core::registry::sha256_hex(families.as_bytes()),
        steps = oci_core::registry::sha256_hex(steps.as_bytes()),
        test_vectors = oci_core::registry::sha256_hex(test_vectors.as_bytes()),
        metadata = oci_core::registry::sha256_hex(metadata.as_bytes()),
        schema = oci_core::registry::sha256_hex(schema.as_bytes())
    )
}

#[derive(Debug, Clone, Copy)]
enum FamilyModel {
    Chromatic { start: f64, end: f64 },
    EarthMuted { start: f64, end: f64 },
    Neutral { hue: f64, max_chroma: f64 },
}

impl FamilyModel {
    fn families_json_fields(self) -> (&'static str, f64, f64) {
        match self {
            Self::Chromatic { start, end } => ("chromatic", start, end),
            Self::EarthMuted { start, end } => ("earth_muted", start, end),
            Self::Neutral { .. } => ("neutral", 0.0, 360.0),
        }
    }

    fn anchor_hue(self, anchor: u8) -> f64 {
        let multiplier = match anchor {
            1 => 1.0 / 6.0,
            2 => 3.0 / 6.0,
            3 => 5.0 / 6.0,
            _ => unreachable!("generator only emits anchors 1..=3"),
        };

        match self {
            Self::Chromatic { start, end } | Self::EarthMuted { start, end } => {
                start + (end - start) * multiplier
            }
            Self::Neutral { hue, .. } => hue,
        }
    }

    fn max_chroma(self, lightness: f64) -> f64 {
        match self {
            Self::Chromatic { .. } => chromatic_max_chroma(lightness),
            Self::EarthMuted { .. } => chromatic_max_chroma(lightness) * 0.55,
            Self::Neutral { max_chroma, .. } => max_chroma,
        }
    }
}

fn family_model(index: u8) -> FamilyModel {
    match index {
        0 => FamilyModel::Chromatic {
            start: 0.0,
            end: 8.0,
        },
        1..=44 => {
            let start = 8.0 + f64::from(index - 1) * (312.0 / 44.0);
            let end = 8.0 + f64::from(index) * (312.0 / 44.0);
            FamilyModel::Chromatic { start, end }
        }
        45 => FamilyModel::Chromatic {
            start: 320.0,
            end: 330.0,
        },
        46 => FamilyModel::Chromatic {
            start: 330.0,
            end: 340.0,
        },
        47 => FamilyModel::Chromatic {
            start: 340.0,
            end: 348.0,
        },
        48 => FamilyModel::Chromatic {
            start: 348.0,
            end: 356.0,
        },
        49 => FamilyModel::Chromatic {
            start: 356.0,
            end: 360.0,
        },
        50 => FamilyModel::EarthMuted {
            start: 340.0,
            end: 352.0,
        },
        51 => FamilyModel::EarthMuted {
            start: 300.0,
            end: 320.0,
        },
        52 => FamilyModel::EarthMuted {
            start: 20.0,
            end: 55.0,
        },
        53 => FamilyModel::EarthMuted {
            start: 18.0,
            end: 48.0,
        },
        54 => FamilyModel::EarthMuted {
            start: 35.0,
            end: 70.0,
        },
        55 => FamilyModel::EarthMuted {
            start: 45.0,
            end: 85.0,
        },
        56 => FamilyModel::EarthMuted {
            start: 65.0,
            end: 105.0,
        },
        57 => FamilyModel::EarthMuted {
            start: 90.0,
            end: 135.0,
        },
        58 => FamilyModel::Neutral {
            hue: 230.0,
            max_chroma: 0.035,
        },
        59 => FamilyModel::Neutral {
            hue: 230.0,
            max_chroma: 0.018,
        },
        60 => FamilyModel::Neutral {
            hue: 0.0,
            max_chroma: 0.012,
        },
        61 => FamilyModel::Neutral {
            hue: 0.0,
            max_chroma: 0.006,
        },
        62 => FamilyModel::Neutral {
            hue: 0.0,
            max_chroma: 0.004,
        },
        63 => FamilyModel::Neutral {
            hue: 0.0,
            max_chroma: 0.004,
        },
        _ => unreachable!("family table is fixed to 64 entries"),
    }
}

fn chromatic_max_chroma(lightness: f64) -> f64 {
    let lightness_window = (1.0 - 0.75 * ((2.0 * lightness) - 1.0).abs()).max(0.25);
    0.40 * lightness_window
}

fn json_escape(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}
