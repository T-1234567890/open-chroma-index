use oci_core::{
    ColorExport, EncodeResult, ExportSet, FloatRgb, InspectResult, Oklab, Oklch, Rgb8,
    SupportEntry, SupportMatrix, SupportStatus, TargetColorSystem,
};

pub fn error_json(code: &str, message: &str) -> String {
    format!(
        "{{\"error\":{{\"code\":\"{}\",\"message\":\"{}\"}}}}",
        escape_json(code),
        escape_json(message)
    )
}

pub fn encode_json(
    input: &str,
    source_space: &str,
    result: &EncodeResult,
    include_exports: bool,
) -> String {
    let mut out = String::new();
    out.push('{');
    push_field(&mut out, "input", &json_string(input), false);
    push_field(&mut out, "sourceSpace", &json_string(source_space), true);
    push_field(
        &mut out,
        "canonical",
        &canonical_json(result.decoded_oklch),
        true,
    );
    push_field(&mut out, "oci", &oci_json(result), true);
    push_field(&mut out, "offset", &offset_json(result.oci_id.offset), true);
    if include_exports {
        push_field(&mut out, "exports", &exports_json(&result.exports), true);
        push_field(
            &mut out,
            "support",
            &support_json(&result.support_matrix),
            true,
        );
    } else {
        push_field(&mut out, "exports", "null", true);
        push_field(&mut out, "support", "[]", true);
    }
    push_field(&mut out, "warnings", "[]", true);
    out.push('}');
    out
}

pub fn inspect_json(input: &str, result: &InspectResult, include_exports: bool) -> String {
    let mut out = String::new();
    out.push('{');
    push_field(
        &mut out,
        "oci",
        &format!(
            "{{\"input\":{},\"short\":{},\"full\":{},\"baseStep\":{}}}",
            json_string(input),
            json_string(&result.short_id),
            json_string(&result.full_id),
            base_step_json(&result.oci_id)
        ),
        false,
    );
    push_field(&mut out, "offset", &offset_json(result.oci_id.offset), true);
    push_field(
        &mut out,
        "canonical",
        &canonical_json(result.canonical_oklch),
        true,
    );
    if include_exports {
        push_field(&mut out, "exports", &exports_json(&result.exports), true);
        push_field(
            &mut out,
            "support",
            &support_json(&result.support_matrix),
            true,
        );
    } else {
        push_field(&mut out, "exports", "null", true);
        push_field(&mut out, "support", "[]", true);
    }
    push_field(&mut out, "warnings", "[]", true);
    out.push('}');
    out
}

pub fn export_json(id: &str, exports: &ExportSet, targets: &[String]) -> String {
    let mut out = String::new();
    out.push('{');
    push_field(&mut out, "oci", &json_string(id), false);
    push_field(
        &mut out,
        "exports",
        &selected_exports_json(exports, targets),
        true,
    );
    out.push('}');
    out
}

pub fn convert_json(input: &str, from: &str, result: &EncodeResult, targets: &[String]) -> String {
    let mut out = String::new();
    out.push('{');
    push_field(&mut out, "input", &json_string(input), false);
    push_field(&mut out, "from", &json_string(from), true);
    push_field(
        &mut out,
        "canonical",
        &canonical_json(result.decoded_oklch),
        true,
    );
    push_field(
        &mut out,
        "oci",
        &format!(
            "{{\"short\":{},\"full\":{}}}",
            json_string(&result.short_id),
            json_string(&result.full_id)
        ),
        true,
    );
    push_field(
        &mut out,
        "exports",
        &selected_exports_json(&result.exports, targets),
        true,
    );
    push_field(
        &mut out,
        "support",
        &support_json(&result.support_matrix),
        true,
    );
    out.push('}');
    out
}

pub fn support_json(matrix: &SupportMatrix) -> String {
    let entries = matrix
        .entries
        .iter()
        .map(support_entry_json)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{entries}]")
}

pub fn exports_json(exports: &ExportSet) -> String {
    selected_exports_json(
        exports,
        &[
            "hex".into(),
            "rgb".into(),
            "hsl".into(),
            "srgb".into(),
            "display-p3".into(),
            "adobe-rgb".into(),
            "rec709".into(),
            "oklch".into(),
            "oklab".into(),
            "css".into(),
            "json-token".into(),
            "cmyk".into(),
        ],
    )
}

pub fn selected_exports_json(exports: &ExportSet, targets: &[String]) -> String {
    let mut fields = Vec::new();
    for target in targets {
        match target.as_str() {
            "hex" => fields.push(format!("\"hex\":{}", export_string_json(&exports.hex))),
            "rgb" => fields.push(format!("\"rgb\":{}", export_rgb8_json(&exports.rgb))),
            "hsl" => fields.push(format!("\"hsl\":{}", export_hsl_json(&exports.hsl))),
            "srgb" => fields.push(format!(
                "\"srgb\":{}",
                export_float_rgb_json(&exports.srgb_float)
            )),
            "display-p3" => fields.push(format!(
                "\"displayP3\":{}",
                export_float_rgb_json(&exports.display_p3_float)
            )),
            "adobe-rgb" => fields.push(format!(
                "\"adobeRgb1998\":{}",
                export_float_rgb_json(&exports.adobe_rgb_1998_float)
            )),
            "rec709" => fields.push(format!(
                "\"rec709\":{}",
                export_float_rgb_json(&exports.rec709_float)
            )),
            "oklch" => fields.push(format!("\"oklch\":{}", oklch_json(exports.oklch))),
            "oklab" => fields.push(format!("\"oklab\":{}", oklab_json(exports.oklab))),
            "css" => fields.push(format!(
                "\"css\":{{\"oklch\":{},\"srgb\":{},\"displayP3\":{}}}",
                json_string(&exports.css.oklch),
                option_string_json(exports.css.srgb.as_deref()),
                option_string_json(exports.css.display_p3.as_deref())
            )),
            "json-token" => fields.push(format!(
                "\"jsonToken\":{{\"model\":\"oklch\",\"l\":{:.6},\"c\":{:.6},\"h\":{:.6}}}",
                exports.oklch.l, exports.oklch.c, exports.oklch.h
            )),
            "swift" => fields.push(format!(
                "\"swift\":{}",
                json_string(&format!(
                    "Color(.displayP3, red: {:.6}, green: {:.6}, blue: {:.6})",
                    exports
                        .display_p3_float
                        .value
                        .map(|rgb| rgb.r)
                        .unwrap_or(0.0),
                    exports
                        .display_p3_float
                        .value
                        .map(|rgb| rgb.g)
                        .unwrap_or(0.0),
                    exports
                        .display_p3_float
                        .value
                        .map(|rgb| rgb.b)
                        .unwrap_or(0.0)
                ))
            )),
            "tailwind" => fields.push(format!(
                "\"tailwind\":{}",
                json_string(&format!("\"oci\": \"{}\"", exports.css.oklch))
            )),
            "cmyk" => fields.push(format!("\"cmyk\":{}", export_string_json(&exports.cmyk))),
            _ => fields.push(format!(
                "\"{}\":{{\"status\":\"unsupported\",\"value\":null}}",
                escape_json(target)
            )),
        }
    }
    format!("{{{}}}", fields.join(","))
}

pub fn registry_info_json(family_count: usize, step_count: usize) -> String {
    format!(
        "{{\"registry\":{{\"version\":\"1-beta\",\"familyCount\":{family_count},\"stepCount\":{step_count}}}}}"
    )
}

pub fn validation_json(valid: bool, target: &str) -> String {
    format!(
        "{{\"target\":{},\"valid\":{}}}",
        json_string(target),
        if valid { "true" } else { "false" }
    )
}

pub fn checksum_json(entries: &[(String, String, bool)]) -> String {
    let files = entries
        .iter()
        .map(|(path, sha, valid)| {
            format!(
                "{{\"path\":{},\"sha256\":{},\"valid\":{}}}",
                json_string(path),
                json_string(sha),
                if *valid { "true" } else { "false" }
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"algorithm\":\"sha256\",\"files\":[{files}]}}")
}

fn oci_json(result: &EncodeResult) -> String {
    format!(
        "{{\"short\":{},\"full\":{},\"precisionShort\":{},\"precisionFull\":{}}}",
        json_string(&base_short(&result.oci_id)),
        json_string(&base_full(&result.oci_id)),
        json_string(&result.short_id),
        json_string(&result.full_id)
    )
}

fn canonical_json(color: Oklch) -> String {
    format!(
        "{{\"oklch\":{},\"oklab\":{}}}",
        oklch_json(color),
        oklab_json(color.to_oklab())
    )
}

fn base_step_json(id: &oci_core::OciId) -> String {
    format!(
        "{{\"family\":{},\"anchor\":{},\"lightness\":{},\"chroma\":{},\"stepNumber\":{}}}",
        json_string(&id.family.to_string()),
        id.step.anchor,
        id.step.lightness,
        id.step.chroma,
        id.step.step_number()
    )
}

fn offset_json(offset: Option<oci_core::OklchOffset>) -> String {
    match offset {
        Some(offset) => format!(
            "{{\"l\":{:.6},\"c\":{:.6},\"h\":{:.6},\"string\":{}}}",
            offset.lightness,
            offset.chroma,
            offset.hue,
            json_string(&offset.to_string())
        ),
        None => "null".to_string(),
    }
}

fn support_entry_json(entry: &SupportEntry) -> String {
    format!(
        "{{\"target\":{},\"status\":{},\"roundTripError\":{},\"note\":{}}}",
        json_string(target_name(entry.target)),
        json_string(status_name(entry.status)),
        entry
            .round_trip_error
            .map(|value| format!("{value:.12}"))
            .unwrap_or_else(|| "null".to_string()),
        option_string_json(entry.note.as_deref())
    )
}

fn export_float_rgb_json(export: &ColorExport<FloatRgb>) -> String {
    let value = export.value.map_or_else(
        || "null".to_string(),
        |rgb| {
            format!(
                "{{\"r\":{:.6},\"g\":{:.6},\"b\":{:.6}}}",
                rgb.r, rgb.g, rgb.b
            )
        },
    );
    export_value_json(&value, export.status, export.round_trip_error)
}

fn export_rgb8_json(export: &ColorExport<Rgb8>) -> String {
    let value = export.value.map_or_else(
        || "null".to_string(),
        |rgb| format!("{{\"r\":{},\"g\":{},\"b\":{}}}", rgb.r, rgb.g, rgb.b),
    );
    export_value_json(&value, export.status, export.round_trip_error)
}

fn export_hsl_json(export: &ColorExport<oci_core::Hsl>) -> String {
    let value = export.value.map_or_else(
        || "null".to_string(),
        |hsl| {
            format!(
                "{{\"h\":{:.6},\"s\":{:.6},\"l\":{:.6}}}",
                hsl.h, hsl.s, hsl.l
            )
        },
    );
    export_value_json(&value, export.status, export.round_trip_error)
}

fn export_string_json(export: &ColorExport<String>) -> String {
    let value = export
        .value
        .as_ref()
        .map_or_else(|| "null".to_string(), |value| json_string(value));
    export_value_json(&value, export.status, export.round_trip_error)
}

fn export_value_json(value: &str, status: SupportStatus, error: Option<f64>) -> String {
    format!(
        "{{\"status\":{},\"value\":{},\"roundTripError\":{}}}",
        json_string(status_name(status)),
        value,
        error
            .map(|value| format!("{value:.12}"))
            .unwrap_or_else(|| "null".to_string())
    )
}

fn oklch_json(color: Oklch) -> String {
    format!(
        "{{\"l\":{:.6},\"c\":{:.6},\"h\":{:.6}}}",
        color.l, color.c, color.h
    )
}

fn oklab_json(color: Oklab) -> String {
    format!(
        "{{\"l\":{:.6},\"a\":{:.6},\"b\":{:.6}}}",
        color.l, color.a, color.b
    )
}

fn push_field(out: &mut String, key: &str, value: &str, comma: bool) {
    if comma {
        out.push(',');
    }
    out.push('"');
    out.push_str(key);
    out.push_str("\":");
    out.push_str(value);
}

fn base_short(id: &oci_core::OciId) -> String {
    let mut base = id.clone();
    base.offset = None;
    base.to_short_string()
}

fn base_full(id: &oci_core::OciId) -> String {
    let mut base = id.clone();
    base.offset = None;
    base.to_full_string()
}

fn option_string_json(value: Option<&str>) -> String {
    value.map_or_else(|| "null".to_string(), json_string)
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", escape_json(value))
}

fn status_name(status: SupportStatus) -> &'static str {
    match status {
        SupportStatus::Supported => "supported",
        SupportStatus::Lossy => "lossy",
        SupportStatus::GamutMapped => "gamut_mapped",
        SupportStatus::Approximation => "approximation",
        SupportStatus::Unsupported => "unsupported",
        SupportStatus::ProfileRequired => "profile_required",
        SupportStatus::ProofRequired => "proof_required",
        SupportStatus::UserSuppliedReference => "user_supplied_reference",
    }
}

fn target_name(target: TargetColorSystem) -> &'static str {
    match target {
        TargetColorSystem::SrgbFloat => "srgb",
        TargetColorSystem::Hex => "hex",
        TargetColorSystem::Rgb8 => "rgb",
        TargetColorSystem::HslSrgb => "hsl",
        TargetColorSystem::DisplayP3Float => "display-p3",
        TargetColorSystem::AdobeRgb1998Float => "adobe-rgb",
        TargetColorSystem::Rec709Float => "rec709",
        TargetColorSystem::Oklch => "oklch",
        TargetColorSystem::Oklab => "oklab",
        TargetColorSystem::Css => "css",
        TargetColorSystem::Json => "json-token",
        TargetColorSystem::Cmyk => "cmyk",
    }
}

pub fn escape_json(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
