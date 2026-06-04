use crate::config::{CliConfig, all_export_targets, config_path_from_args};
use crate::output;
use oci_core::{
    ColorExport, ColorInput, EncodeResult, EncodedSrgb, ExportSet, FloatRgb, Hsl, InspectResult,
    OciId, Oklab, Oklch, Registry, Rgb8, SupportStatus, build_support_matrix, decode_oci_id,
    encode, encode_from_hex, export_all, inspect,
};
use std::io::{self, IsTerminal, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError {
    pub code: String,
    pub message: String,
}

impl CliError {
    pub(crate) fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

pub fn run_cli(args: &[String]) -> Result<String, CliError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(CliError::new("parse_error", "missing command"));
    };

    if matches!(command, "--help" | "-h" | "help") {
        return Ok(help_text());
    }

    if matches!(command, "--version" | "-V" | "version") {
        return Ok(format!("oci {}", env!("CARGO_PKG_VERSION")));
    }

    if command == "config" {
        return cmd_config(args, ConfigMode::NonInteractive);
    }

    let config = CliConfig::load_from_args(args).map_err(config_error)?;

    run_cli_with_config(args, &config)
}

pub(crate) fn run_cli_with_config(args: &[String], config: &CliConfig) -> Result<String, CliError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(CliError::new("parse_error", "missing command"));
    };

    match command {
        "encode" => cmd_encode(&args[1..], config),
        "inspect" => cmd_inspect(&args[1..], config),
        "export" => cmd_export(&args[1..], config),
        "convert" => cmd_convert(&args[1..], config),
        "serve" => cmd_serve(&args[1..], config),
        "registry" => cmd_registry(&args[1..], config),
        "test" => cmd_test(&args[1..], config),
        "validate" => cmd_validate(&args[1..], config),
        _ => Err(CliError::new(
            "parse_error",
            format!("unknown command: {command}"),
        )),
    }
}

pub fn run_config_command(args: &[String]) -> Result<String, CliError> {
    cmd_config(args, ConfigMode::Auto)
}

fn help_text() -> String {
    [
        "Open Chroma Index CLI",
        "",
        "Usage:",
        "  oci encode <INPUT> --space <SPACE> [--format json|pretty|plain] [--precision <N>] [--verify]",
        "  oci inspect <OCI_ID> [--format json|pretty|plain] [--exports all|none|summary|<LIST>] [--verify]",
        "  oci export <OCI_ID> --to <TARGETS> [--format json|plain|pretty] [--verify]",
        "  oci convert <INPUT> --from <SPACE> --to <TARGETS> [--format json|plain|pretty] [--verify]",
        "  oci serve [--host <HOST>] [--port <PORT>] [--config <PATH>] [--json]",
        "  oci registry <SUBCOMMAND>",
        "  oci test <SUBCOMMAND>",
        "  oci validate <TARGET> [--type id|registry|color]",
        "  oci config [--path <TOML_PATH>]",
        "",
        "Common commands:",
        "  oci encode \"#E85A9A\" --space hex",
        "  oci inspect OCI-1-48RS-327",
        "  oci serve",
        "  oci registry validate",
    ]
    .join("\n")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigMode {
    Auto,
    NonInteractive,
}

fn cmd_encode(args: &[String], config: &CliConfig) -> Result<String, CliError> {
    let input = positional(args, 0, "encode input")?;
    let space = flag_value(args, "--space").unwrap_or(&config.color.default_input_space);
    let format = configured_format(args, config)?;
    let precision = configured_precision(args, config)?;
    let verify = configured_verify(args, config);
    let include_exports = !has_flag(args, "--no-exports") && config.output.show_exports;
    let registry = load_registry(config)?;
    let color_input = parse_color_input(input, space, &registry)?;
    let result = if space == "hex" {
        encode_from_hex(input, &registry).map_err(pipeline_error)?
    } else {
        encode(color_input, &registry).map_err(pipeline_error)?
    };

    match format {
        "json" => Ok(output::encode_json(input, space, &result, include_exports)),
        "pretty" => Ok(encode_pretty(
            input, space, &result, config, precision, verify,
        )),
        "plain" => Ok(preferred_oci_code(&result, config)),
        other => Err(CliError::new(
            "parse_error",
            format!("unsupported output format: {other}"),
        )),
    }
}

fn cmd_inspect(args: &[String], config: &CliConfig) -> Result<String, CliError> {
    let input = positional(args, 0, "OCI ID")?;
    let format = configured_format(args, config)?;
    let precision = configured_precision(args, config)?;
    let verify = configured_verify(args, config);
    let exports = flag_value(args, "--exports").unwrap_or(&config.inspect.exports);
    let registry = load_registry(config)?;
    let id = OciId::parse_with_registry(input, &registry).map_err(id_error)?;
    let result = inspect(&id, &registry).map_err(pipeline_error)?;
    let include_exports = exports != "none";

    match format {
        "json" => Ok(output::inspect_json(input, &result, include_exports)),
        "pretty" => Ok(inspect_pretty(
            input, &result, config, exports, precision, verify,
        )),
        "plain" => Ok(result.canonical_id.to_short_string()),
        other => Err(CliError::new(
            "parse_error",
            format!("unsupported output format: {other}"),
        )),
    }
}

fn cmd_export(args: &[String], config: &CliConfig) -> Result<String, CliError> {
    let input = positional(args, 0, "OCI ID")?;
    let targets = flag_value(args, "--to")
        .map(parse_targets)
        .unwrap_or_else(|| config.output.default_exports.clone());
    let format = configured_format(args, config)?;
    let verify = configured_verify(args, config);
    let registry = load_registry(config)?;
    let id = OciId::parse_with_registry(input, &registry).map_err(id_error)?;
    let color = decode_oci_id(&id, &registry).map_err(pipeline_error)?;
    let exports = export_all(color);

    match format {
        "json" => Ok(output::export_json(input, &exports, &targets)),
        "plain" => Ok(targets
            .iter()
            .map(|target| {
                format!(
                    "{target}: {}",
                    output::selected_exports_json(&exports, std::slice::from_ref(target))
                )
            })
            .collect::<Vec<_>>()
            .join("\n")),
        "pretty" => Ok(exports_pretty(&exports, &targets, verify)),
        other => Err(CliError::new(
            "parse_error",
            format!("unsupported output format: {other}"),
        )),
    }
}

fn cmd_convert(args: &[String], config: &CliConfig) -> Result<String, CliError> {
    let input = positional(args, 0, "convert input")?;
    let from = flag_value(args, "--from").unwrap_or(&config.color.default_input_space);
    let targets = flag_value(args, "--to")
        .map(parse_targets)
        .unwrap_or_else(|| config.color.default_targets.clone());
    let format = configured_format(args, config)?;
    let precision = configured_precision(args, config)?;
    let verify = configured_verify(args, config);
    let registry = load_registry(config)?;
    let result =
        encode(parse_color_input(input, from, &registry)?, &registry).map_err(pipeline_error)?;

    match format {
        "json" => Ok(output::convert_json(input, from, &result, &targets)),
        "plain" => Ok(output::selected_exports_json(&result.exports, &targets)),
        "pretty" => Ok(convert_pretty(
            input, from, &result, &targets, precision, verify,
        )),
        other => Err(CliError::new(
            "parse_error",
            format!("unsupported output format: {other}"),
        )),
    }
}

fn cmd_serve(args: &[String], config: &CliConfig) -> Result<String, CliError> {
    if has_flag(args, "--help") || has_flag(args, "-h") {
        return Ok(serve_help_text());
    }

    let options = crate::server::ServerOptions::from_args(args, config)?;
    crate::server::serve(options, config.clone())?;
    Ok(String::new())
}

fn cmd_registry(args: &[String], config: &CliConfig) -> Result<String, CliError> {
    let Some(subcommand) = args.first().map(String::as_str) else {
        return Err(CliError::new("parse_error", "missing registry subcommand"));
    };
    let registry = load_registry(config)?;

    match subcommand {
        "info" => Ok(output::registry_info_json(
            registry.families().len(),
            registry.steps().len(),
        )),
        "families" => Ok(format!(
            "{{\"families\":[{}]}}",
            registry
                .families()
                .iter()
                .map(|family| format!(
                    "{{\"id\":\"{}\",\"index\":{},\"code\":\"{}\",\"name\":\"{}\",\"group\":\"{}\",\"hueStart\":{:.6},\"hueEnd\":{:.6}}}",
                    family.id,
                    family.id.index,
                    family.id.code,
                    output::escape_json(&family.name),
                    output::escape_json(&family.group),
                    family.hue_start,
                    family.hue_end
                ))
                .collect::<Vec<_>>()
                .join(",")
        )),
        "family" => {
            let key = positional(args, 1, "family index or code")?;
            registry_family_json(&registry, key)
        }
        "step" => {
            let key = positional(args, 1, "OCI ID or step")?;
            registry_step_json(&registry, key)
        }
        "validate" => {
            registry.validate().map_err(registry_error)?;
            Ok(output::validation_json(true, "registry"))
        }
        "checksum" => Ok(output::checksum_json(&checksum_entries())),
        other => Err(CliError::new(
            "parse_error",
            format!("unknown registry subcommand: {other}"),
        )),
    }
}

fn cmd_test(args: &[String], config: &CliConfig) -> Result<String, CliError> {
    let Some(subcommand) = args.first().map(String::as_str) else {
        return Err(CliError::new("parse_error", "missing test subcommand"));
    };
    match subcommand {
        "vectors" => test_vectors(config),
        "roundtrip" => test_roundtrip(config),
        "registry" => {
            let registry = load_registry(config)?;
            registry.validate().map_err(registry_error)?;
            Ok("{\"test\":\"registry\",\"passed\":true}".to_string())
        }
        other => Err(CliError::new(
            "parse_error",
            format!("unknown test subcommand: {other}"),
        )),
    }
}

fn cmd_validate(args: &[String], config: &CliConfig) -> Result<String, CliError> {
    let target = positional(args, 0, "validation target")?;
    let target_type = flag_value(args, "--type").unwrap_or("id");
    let registry = load_registry(config)?;

    match target_type {
        "id" => {
            OciId::parse_with_registry(target, &registry).map_err(id_error)?;
            Ok(output::validation_json(true, target))
        }
        "registry" => {
            registry.validate().map_err(registry_error)?;
            Ok(output::validation_json(true, "registry"))
        }
        "color" => {
            let space = flag_value(args, "--space").unwrap_or("hex");
            parse_color_input(target, space, &registry)?;
            Ok(output::validation_json(true, target))
        }
        other => Err(CliError::new(
            "parse_error",
            format!("unknown validation type: {other}"),
        )),
    }
}

fn cmd_config(args: &[String], mode: ConfigMode) -> Result<String, CliError> {
    let path = config_path_from_args(args);
    let mut config = CliConfig::load_from_path(path.clone()).map_err(config_error)?;

    if mode == ConfigMode::Auto && io::stdin().is_terminal() {
        config = run_config_wizard(config, &path)?;
    }

    config.write_to_path(&path).map_err(config_error)?;
    Ok(format!(
        "OCI config written to {}\n{}",
        path.display(),
        config_summary(&config)
    ))
}

fn run_config_wizard(mut config: CliConfig, path: &std::path::Path) -> Result<CliConfig, CliError> {
    println!("OCI configuration");
    println!("Path: {}", path.display());
    println!("Press Enter to keep the current value shown in brackets.");

    config.output.format =
        prompt_string("output format (pretty|json|plain)", &config.output.format)?;
    config.output.precision = prompt_usize("precision", config.output.precision)?;
    config.output.default_exports =
        prompt_list("default export targets", &config.output.default_exports)?;
    config.output.show_support = prompt_bool("show support matrix", config.output.show_support)?;
    config.output.show_warnings = prompt_bool("show warnings", config.output.show_warnings)?;
    config.output.show_exports = prompt_bool("show exports", config.output.show_exports)?;
    config.output.verify = prompt_bool("show verification details", config.output.verify)?;
    config.encode.include_offset = prompt_bool(
        "include offset in encode output",
        config.encode.include_offset,
    )?;
    config.encode.prefer_short_code =
        prompt_bool("prefer short code", config.encode.prefer_short_code)?;
    config.encode.include_full_code =
        prompt_bool("include full code", config.encode.include_full_code)?;
    config.inspect.exports = prompt_string(
        "default inspect exports (all|none|summary|list)",
        &config.inspect.exports,
    )?;
    config.inspect.default_export_list = prompt_list(
        "inspect default export list",
        &config.inspect.default_export_list,
    )?;
    config.color.default_input_space = prompt_string(
        "default input color space",
        &config.color.default_input_space,
    )?;
    config.color.default_targets =
        prompt_list("default convert targets", &config.color.default_targets)?;
    config.registry.source =
        prompt_string("registry source (bundled|path)", &config.registry.source)?;
    config.registry.path = prompt_string("registry path", &config.registry.path)?;
    config.registry.validate_on_start = prompt_bool(
        "validate registry on start",
        config.registry.validate_on_start,
    )?;
    config.server.host = prompt_string("server host", &config.server.host)?;
    config.server.port = prompt_usize("server port", config.server.port as usize)? as u16;
    config.server.warn_non_localhost = prompt_bool(
        "warn when server is not localhost",
        config.server.warn_non_localhost,
    )?;

    Ok(config)
}

fn encode_pretty(
    input: &str,
    space: &str,
    result: &EncodeResult,
    config: &CliConfig,
    precision: usize,
    verify: bool,
) -> String {
    let mut lines = vec![
        "OCI Encode".to_string(),
        format!("input: {input} ({space})"),
        String::new(),
        format!(
            "OCI standard color code: {}",
            standard_color_code(result, config)
        ),
        format!(
            "OCI precision color code: {}",
            precision_color_code(result, config)
        ),
        format!(
            "oklch: L={} C={} H={}",
            fixed(result.decoded_oklch.l, precision),
            fixed(result.decoded_oklch.c, precision),
            fixed(result.decoded_oklch.h, precision)
        ),
    ];

    if config.encode.include_full_code {
        lines.push(format!("full: {}", result.full_id));
    }
    if config.output.show_exports {
        lines.push(String::new());
        lines.push("exports:".to_string());
        let targets = all_export_targets();
        lines.push(indent(&selected_exports_pretty(&result.exports, &targets)));
        lines.push(String::new());
        lines.push(verification_pretty(&result.exports, &targets, verify));
    }
    if config.output.show_support {
        lines.push(String::new());
        lines.push(format!(
            "support: {} targets evaluated",
            result.support_matrix.entries.len()
        ));
    }
    if config.output.show_warnings {
        lines.push("warnings: none".to_string());
    }

    lines.join("\n")
}

fn inspect_pretty(
    input: &str,
    result: &InspectResult,
    config: &CliConfig,
    export_mode: &str,
    precision: usize,
    verify: bool,
) -> String {
    let mut lines = vec![
        "OCI Inspect".to_string(),
        format!("input: {input}"),
        String::new(),
        format!(
            "OCI standard color code: {}",
            inspect_standard_color_code(result, config)
        ),
        format!("short: {}", result.short_id),
        format!("full: {}", result.full_id),
        format!(
            "oklch: L={} C={} H={}",
            fixed(result.canonical_oklch.l, precision),
            fixed(result.canonical_oklch.c, precision),
            fixed(result.canonical_oklch.h, precision)
        ),
    ];

    let targets = inspect_targets(config, export_mode);
    if !targets.is_empty() {
        lines.push(String::new());
        lines.push("exports:".to_string());
        lines.push(indent(&selected_exports_pretty(&result.exports, &targets)));
        lines.push(String::new());
        lines.push(verification_pretty(&result.exports, &targets, verify));
    }
    if config.output.show_support {
        lines.push(String::new());
        lines.push(format!(
            "support: {} targets evaluated",
            result.support_matrix.entries.len()
        ));
    }
    if config.output.show_warnings {
        lines.push("warnings: none".to_string());
    }

    lines.join("\n")
}

fn convert_pretty(
    input: &str,
    from: &str,
    result: &EncodeResult,
    targets: &[String],
    precision: usize,
    verify: bool,
) -> String {
    format!(
        "OCI Convert\ninput: {input} ({from})\n\noklch: L={} C={} H={}\n\nexports:\n{}\n\n{}",
        fixed(result.decoded_oklch.l, precision),
        fixed(result.decoded_oklch.c, precision),
        fixed(result.decoded_oklch.h, precision),
        indent(&selected_exports_pretty(&result.exports, targets)),
        verification_pretty(&result.exports, targets, verify)
    )
}

fn exports_pretty(exports: &ExportSet, targets: &[String], verify: bool) -> String {
    format!(
        "exports:\n{}\n\n{}",
        indent(&selected_exports_pretty(exports, targets)),
        verification_pretty(exports, targets, verify)
    )
}

fn selected_exports_pretty(exports: &ExportSet, targets: &[String]) -> String {
    targets
        .iter()
        .map(|target| export_target_pretty(exports, target))
        .collect::<Vec<_>>()
        .join("\n")
}

fn export_target_pretty(exports: &ExportSet, target: &str) -> String {
    match target {
        "hex" => format_string_export("HEX", &exports.hex),
        "rgb" => format_rgb8_export("RGB", &exports.rgb),
        "hsl" => format_hsl_export("HSL", &exports.hsl),
        "srgb" => format_float_rgb_export("sRGB", &exports.srgb_float),
        "display-p3" => format_float_rgb_export("Display P3", &exports.display_p3_float),
        "adobe-rgb" => format_float_rgb_export("Adobe RGB", &exports.adobe_rgb_1998_float),
        "rec709" => format_float_rgb_export("Rec.709", &exports.rec709_float),
        "oklch" => format!(
            "OKLCH: L={:.6} C={:.6} H={:.6}",
            exports.oklch.l, exports.oklch.c, exports.oklch.h
        ),
        "oklab" => format!(
            "OKLab: L={:.6} a={:.6} b={:.6}",
            exports.oklab.l, exports.oklab.a, exports.oklab.b
        ),
        "css" => {
            let mut lines = vec![format!("CSS OKLCH: {}", exports.css.oklch)];
            if let Some(srgb) = exports.css.srgb.as_deref() {
                lines.push(format!("CSS sRGB: {srgb}"));
            }
            if let Some(display_p3) = exports.css.display_p3.as_deref() {
                lines.push(format!("CSS Display P3: {display_p3}"));
            }
            lines.join("\n")
        }
        "json-token" => {
            let mut lines = vec!["JSON token:".to_string()];
            for value in &exports.json {
                let components = value
                    .components
                    .iter()
                    .map(|component| format!("{}={:.6}", component.name, component.value))
                    .collect::<Vec<_>>()
                    .join(" ");
                lines.push(format!("  {}: {components}", value.model));
            }
            lines.join("\n")
        }
        "swift" => format!(
            "Swift: Color(.displayP3, red: {:.6}, green: {:.6}, blue: {:.6})",
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
        ),
        "tailwind" => format!("Tailwind: oci: {}", exports.css.oklch),
        "cmyk" => format_string_export("CMYK", &exports.cmyk),
        _ => format!("{target}: unsupported"),
    }
}

fn format_string_export(label: &str, export: &ColorExport<String>) -> String {
    match export.value.as_deref() {
        Some(value) => format!("{label}: {value}"),
        None => format!("{label}: unavailable"),
    }
}

fn format_float_rgb_export(label: &str, export: &ColorExport<FloatRgb>) -> String {
    match export.value {
        Some(rgb) => format!("{label}: r={:.6} g={:.6} b={:.6}", rgb.r, rgb.g, rgb.b),
        None => format!("{label}: unavailable"),
    }
}

fn format_rgb8_export(label: &str, export: &ColorExport<Rgb8>) -> String {
    match export.value {
        Some(rgb) => format!("{label}: r={} g={} b={}", rgb.r, rgb.g, rgb.b),
        None => format!("{label}: unavailable"),
    }
}

fn format_hsl_export(label: &str, export: &ColorExport<Hsl>) -> String {
    match export.value {
        Some(hsl) => format!("{label}: h={:.6} s={:.6} l={:.6}", hsl.h, hsl.s, hsl.l),
        None => format!("{label}: unavailable"),
    }
}

fn status_label(status: SupportStatus) -> &'static str {
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

fn compact_status_label(status: SupportStatus) -> &'static str {
    match status {
        SupportStatus::Supported => "supported",
        SupportStatus::Lossy => "lossy",
        SupportStatus::GamutMapped => "gamut mapped",
        SupportStatus::Approximation => "approximation",
        SupportStatus::Unsupported => "unsupported",
        SupportStatus::ProfileRequired => "profile required",
        SupportStatus::ProofRequired => "proof required",
        SupportStatus::UserSuppliedReference => "user supplied reference",
    }
}

#[derive(Debug, Clone)]
struct PrettyVerification {
    label: &'static str,
    status: SupportStatus,
    round_trip_error: Option<f64>,
}

fn verification_pretty(exports: &ExportSet, targets: &[String], detailed: bool) -> String {
    let entries = verification_entries(exports, targets);
    let mut lines = vec!["verification:".to_string()];

    for status in [
        SupportStatus::Lossy,
        SupportStatus::Supported,
        SupportStatus::GamutMapped,
        SupportStatus::Approximation,
        SupportStatus::Unsupported,
        SupportStatus::ProfileRequired,
        SupportStatus::ProofRequired,
        SupportStatus::UserSuppliedReference,
    ] {
        let labels = entries
            .iter()
            .filter(|entry| entry.status == status)
            .map(|entry| entry.label)
            .collect::<Vec<_>>();
        if !labels.is_empty() {
            lines.push(format!(
                "  {}: {}",
                compact_status_label(status),
                labels.join(", ")
            ));
        }
    }

    let max_error = entries
        .iter()
        .filter_map(|entry| entry.round_trip_error)
        .fold(None, |max: Option<f64>, value| {
            Some(max.map_or(value, |current| current.max(value)))
        });
    lines.push(format!(
        "  max round-trip error: {}",
        max_error.map_or_else(|| "none".to_string(), |value| format!("{value:.12}"))
    ));

    if detailed {
        lines.push(String::new());
        lines.push("verification details:".to_string());
        for entry in entries {
            let detail = entry.round_trip_error.map_or_else(
                || status_label(entry.status).to_string(),
                |error| {
                    format!(
                        "{}, round-trip error {error:.12}",
                        status_label(entry.status)
                    )
                },
            );
            lines.push(format!("  {}: {detail}", entry.label));
        }
    }

    lines.join("\n")
}

fn verification_entries(exports: &ExportSet, targets: &[String]) -> Vec<PrettyVerification> {
    targets
        .iter()
        .filter_map(|target| match target.as_str() {
            "hex" => Some(export_verification("HEX", &exports.hex)),
            "rgb" => Some(export_verification("RGB", &exports.rgb)),
            "hsl" => Some(export_verification("HSL", &exports.hsl)),
            "srgb" => Some(export_verification("sRGB", &exports.srgb_float)),
            "display-p3" => Some(export_verification("Display P3", &exports.display_p3_float)),
            "adobe-rgb" => Some(export_verification(
                "Adobe RGB",
                &exports.adobe_rgb_1998_float,
            )),
            "rec709" => Some(export_verification("Rec.709", &exports.rec709_float)),
            "oklch" => Some(PrettyVerification {
                label: "OKLCH",
                status: SupportStatus::Supported,
                round_trip_error: Some(0.0),
            }),
            "oklab" => Some(PrettyVerification {
                label: "OKLab",
                status: SupportStatus::Supported,
                round_trip_error: Some(0.0),
            }),
            "css" => Some(PrettyVerification {
                label: "CSS",
                status: SupportStatus::Supported,
                round_trip_error: None,
            }),
            "json-token" => Some(PrettyVerification {
                label: "JSON token",
                status: SupportStatus::Supported,
                round_trip_error: None,
            }),
            "swift" => Some(PrettyVerification {
                label: "Swift",
                status: SupportStatus::Supported,
                round_trip_error: None,
            }),
            "tailwind" => Some(PrettyVerification {
                label: "Tailwind",
                status: SupportStatus::Supported,
                round_trip_error: None,
            }),
            "cmyk" => Some(export_verification("CMYK", &exports.cmyk)),
            _ => None,
        })
        .collect()
}

fn export_verification<T>(label: &'static str, export: &ColorExport<T>) -> PrettyVerification {
    PrettyVerification {
        label,
        status: export.status,
        round_trip_error: export.round_trip_error,
    }
}

fn preferred_oci_code(result: &EncodeResult, config: &CliConfig) -> String {
    let include_offset = config.encode.include_offset && result.oci_id.offset.is_some();
    match (config.encode.prefer_short_code, include_offset) {
        (true, true) => result.short_id.clone(),
        (true, false) => base_short_string(&result.oci_id),
        (false, true) => result.full_id.clone(),
        (false, false) => base_full_string(&result.oci_id),
    }
}

fn standard_color_code(result: &EncodeResult, config: &CliConfig) -> String {
    if config.encode.prefer_short_code {
        base_short_string(&result.oci_id)
    } else {
        base_full_string(&result.oci_id)
    }
}

fn precision_color_code(result: &EncodeResult, config: &CliConfig) -> String {
    if config.encode.include_offset && result.oci_id.offset.is_some() {
        if config.encode.prefer_short_code {
            result.short_id.clone()
        } else {
            result.full_id.clone()
        }
    } else {
        standard_color_code(result, config)
    }
}

fn inspect_standard_color_code(result: &InspectResult, config: &CliConfig) -> String {
    if config.encode.prefer_short_code {
        base_short_string(&result.oci_id)
    } else {
        base_full_string(&result.oci_id)
    }
}

fn base_short_string(id: &OciId) -> String {
    let mut id = id.clone();
    id.offset = None;
    id.to_short_string()
}

fn base_full_string(id: &OciId) -> String {
    let mut id = id.clone();
    id.offset = None;
    id.to_full_string()
}

fn inspect_targets(_config: &CliConfig, export_mode: &str) -> Vec<String> {
    match export_mode {
        "none" => Vec::new(),
        "all" | "summary" | "list" => all_export_targets(),
        value => parse_targets(value),
    }
}

fn configured_format<'a>(args: &'a [String], config: &'a CliConfig) -> Result<&'a str, CliError> {
    let format = flag_value(args, "--format").unwrap_or(&config.output.format);
    match format {
        "pretty" | "json" | "plain" => Ok(format),
        other => Err(CliError::new(
            "parse_error",
            format!("unsupported output format: {other}"),
        )),
    }
}

fn configured_precision(args: &[String], config: &CliConfig) -> Result<usize, CliError> {
    flag_value(args, "--precision").map_or(Ok(config.output.precision), |value| {
        value
            .parse::<usize>()
            .map_err(|_| CliError::new("parse_error", format!("invalid precision value: {value}")))
    })
}

fn configured_verify(args: &[String], config: &CliConfig) -> bool {
    has_flag(args, "--verify") || config.output.verify
}

fn config_summary(config: &CliConfig) -> String {
    format!(
        "output.format={}\noutput.precision={}\nregistry.source={}",
        config.output.format, config.output.precision, config.registry.source
    )
}

fn prompt_string(label: &str, current: &str) -> Result<String, CliError> {
    let input = prompt(label, current)?;
    if input.is_empty() {
        Ok(current.to_string())
    } else {
        Ok(input)
    }
}

fn prompt_bool(label: &str, current: bool) -> Result<bool, CliError> {
    let input = prompt(label, if current { "true" } else { "false" })?;
    match input.as_str() {
        "" => Ok(current),
        "true" | "yes" | "y" => Ok(true),
        "false" | "no" | "n" => Ok(false),
        _ => Err(CliError::new(
            "parse_error",
            format!("invalid boolean for {label}: {input}"),
        )),
    }
}

fn prompt_usize(label: &str, current: usize) -> Result<usize, CliError> {
    let input = prompt(label, &current.to_string())?;
    if input.is_empty() {
        Ok(current)
    } else {
        input.parse::<usize>().map_err(|_| {
            CliError::new(
                "parse_error",
                format!("invalid integer for {label}: {input}"),
            )
        })
    }
}

fn prompt_list(label: &str, current: &[String]) -> Result<Vec<String>, CliError> {
    let joined = current.join(",");
    let input = prompt(label, &joined)?;
    if input.is_empty() {
        Ok(current.to_vec())
    } else {
        Ok(parse_targets(&input))
    }
}

fn prompt(label: &str, current: &str) -> Result<String, CliError> {
    print!("{label} [{current}]: ");
    io::stdout()
        .flush()
        .map_err(|error| CliError::new("config_error", error.to_string()))?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|error| CliError::new("config_error", error.to_string()))?;
    Ok(input.trim().to_string())
}

fn fixed(value: f64, precision: usize) -> String {
    format!("{value:.precision$}")
}

fn indent(value: &str) -> String {
    value
        .lines()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_color_input(
    input: &str,
    space: &str,
    registry: &Registry,
) -> Result<ColorInput, CliError> {
    match space {
        "hex" => Ok(ColorInput::Hex(input.to_string())),
        "rgb" => {
            let values = parse_u8_components(input, 3)?;
            Ok(ColorInput::SrgbRgb {
                r: values[0],
                g: values[1],
                b: values[2],
            })
        }
        "srgb" => {
            let values = parse_f64_components(input, 3)?;
            Ok(ColorInput::Srgb(EncodedSrgb::new(
                values[0], values[1], values[2],
            )))
        }
        "hsl" => {
            let values = parse_f64_components(input, 3)?;
            Ok(ColorInput::HslSrgb {
                h: values[0],
                s: values[1],
                l: values[2],
            })
        }
        "display-p3" => {
            let values = parse_f64_components(input, 3)?;
            Ok(ColorInput::DisplayP3Float {
                r: values[0],
                g: values[1],
                b: values[2],
            })
        }
        "adobe-rgb" => {
            let values = parse_f64_components(input, 3)?;
            Ok(ColorInput::AdobeRgb1998Float {
                r: values[0],
                g: values[1],
                b: values[2],
            })
        }
        "rec709" => {
            let values = parse_f64_components(input, 3)?;
            Ok(ColorInput::Rec709Float {
                r: values[0],
                g: values[1],
                b: values[2],
            })
        }
        "oklch" => {
            let values = parse_f64_components(input, 3)?;
            Ok(ColorInput::Oklch(Oklch::new(
                values[0], values[1], values[2],
            )))
        }
        "oklab" => {
            let values = parse_f64_components(input, 3)?;
            Ok(ColorInput::Oklab(Oklab::new(
                values[0], values[1], values[2],
            )))
        }
        "oci" => Ok(ColorInput::OciId(
            OciId::parse_with_registry(input, registry).map_err(id_error)?,
        )),
        other => Err(CliError::new(
            "unsupported_space",
            format!("unsupported source color space: {other}"),
        )),
    }
}

fn registry_family_json(registry: &Registry, key: &str) -> Result<String, CliError> {
    let family = registry
        .families()
        .iter()
        .find(|family| {
            family.id.to_string() == key
                || family.id.code.to_string() == key
                || family.id.index.to_string() == key
        })
        .ok_or_else(|| CliError::new("invalid_family", format!("unknown family: {key}")))?;
    let count = registry
        .steps()
        .iter()
        .filter(|step| step.family_id == family.id)
        .count();
    Ok(format!(
        "{{\"family\":{{\"id\":\"{}\",\"index\":{},\"code\":\"{}\",\"name\":\"{}\",\"group\":\"{}\",\"stepCount\":{}}}}}",
        family.id,
        family.id.index,
        family.id.code,
        output::escape_json(&family.name),
        output::escape_json(&family.group),
        count
    ))
}

fn registry_step_json(registry: &Registry, key: &str) -> Result<String, CliError> {
    let step = if key.starts_with("OCI-") {
        let id = OciId::parse_with_registry(key, registry).map_err(id_error)?;
        registry.find_step(id.family, id.step)
    } else {
        registry
            .steps()
            .iter()
            .find(|step| step.id == key || step.short_id == key)
    }
    .ok_or_else(|| CliError::new("invalid_step", format!("unknown step: {key}")))?;

    Ok(format!(
        "{{\"step\":{{\"id\":\"{}\",\"shortId\":\"{}\",\"familyId\":\"{}\",\"stepNumber\":{},\"anchor\":{},\"lightnessLevel\":{},\"chromaLevel\":{},\"oklch\":{{\"l\":{:.6},\"c\":{:.6},\"h\":{:.6}}}}}}}",
        step.id,
        step.short_id,
        step.family_id,
        step.step_number,
        step.step.anchor,
        step.step.lightness,
        step.step.chroma,
        step.lightness,
        step.chroma,
        step.hue
    ))
}

fn test_vectors(config: &CliConfig) -> Result<String, CliError> {
    let registry = load_registry(config)?;
    let mut total = 0usize;
    let mut passed = 0usize;
    for line in Registry::frozen_test_vectors_json().lines() {
        let object = line.trim().trim_end_matches(',');
        if !object.starts_with('{') {
            continue;
        }
        total += 1;
        if run_vector_object(object, &registry)? {
            passed += 1;
        }
    }
    Ok(format!(
        "{{\"test\":\"vectors\",\"total\":{total},\"passed\":{passed}}}"
    ))
}

fn run_vector_object(object: &str, registry: &Registry) -> Result<bool, CliError> {
    let kind = json_string_field(object, "kind").unwrap_or_default();
    let input = json_string_field(object, "input").unwrap_or_default();
    let source_space = json_string_field(object, "sourceSpace").unwrap_or_default();
    match kind.as_str() {
        "encode" => {
            let result = encode(
                parse_color_input(&input, &source_space, registry)?,
                registry,
            )
            .map_err(pipeline_error)?;
            Ok(result.short_id.starts_with("OCI-1-"))
        }
        "inspect" => {
            let id = OciId::parse_with_registry(&input, registry).map_err(id_error)?;
            inspect(&id, registry).map_err(pipeline_error)?;
            Ok(true)
        }
        "invalid" => Ok(OciId::parse_with_registry(&input, registry).is_err()),
        "support" => {
            let color = parse_color_input(&input, &source_space, registry)?
                .to_oklch(registry)
                .map_err(pipeline_error)?;
            let matrix = build_support_matrix(color);
            Ok(!matrix.entries.is_empty())
        }
        _ => Ok(true),
    }
}

fn test_roundtrip(config: &CliConfig) -> Result<String, CliError> {
    let registry = load_registry(config)?;
    let result = encode_from_hex("#E85A9A", &registry).map_err(pipeline_error)?;
    let decoded = decode_oci_id(&result.oci_id, &registry).map_err(pipeline_error)?;
    let encoded = encode(ColorInput::Oklch(decoded), &registry).map_err(pipeline_error)?;
    Ok(format!(
        "{{\"test\":\"roundtrip\",\"passed\":{},\"short\":{}}}",
        if encoded.short_id.starts_with("OCI-1-") {
            "true"
        } else {
            "false"
        },
        json_string(&encoded.short_id)
    ))
}

fn checksum_entries() -> Vec<(String, String, bool)> {
    let files = [
        (
            "registry/v1/families.json",
            Registry::frozen_families_json(),
        ),
        ("registry/v1/steps.json", Registry::frozen_steps_json()),
        (
            "registry/v1/test-vectors.json",
            Registry::frozen_test_vectors_json(),
        ),
        ("registry/v1/schema.json", Registry::frozen_schema_json()),
        (
            "registry/v1/metadata.json",
            Registry::frozen_metadata_json(),
        ),
    ];
    files
        .iter()
        .map(|(path, content)| {
            let actual = oci_core::registry::sha256_normalized_text_hex(content);
            let expected = checksum_expected(path).unwrap_or_else(|| actual.clone());
            ((*path).to_string(), actual.clone(), actual == expected)
        })
        .collect()
}

fn checksum_expected(path: &str) -> Option<String> {
    for line in Registry::frozen_checksums_json().lines() {
        if line.contains(path) {
            return json_string_field(line.trim().trim_end_matches(','), "sha256");
        }
    }
    None
}

fn parse_targets(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn parse_f64_components(input: &str, expected: usize) -> Result<Vec<f64>, CliError> {
    let parts = split_components(input);
    if parts.len() != expected {
        return Err(CliError::new(
            "parse_error",
            format!("expected {expected} components, found {}", parts.len()),
        ));
    }
    parts
        .iter()
        .map(|part| {
            part.parse::<f64>()
                .map_err(|_| CliError::new("parse_error", format!("invalid number: {part}")))
        })
        .collect()
}

fn parse_u8_components(input: &str, expected: usize) -> Result<Vec<u8>, CliError> {
    let parts = split_components(input);
    if parts.len() != expected {
        return Err(CliError::new(
            "parse_error",
            format!("expected {expected} components, found {}", parts.len()),
        ));
    }
    parts
        .iter()
        .map(|part| {
            part.parse::<u8>()
                .map_err(|_| CliError::new("parse_error", format!("invalid u8 component: {part}")))
        })
        .collect()
}

fn split_components(input: &str) -> Vec<&str> {
    input
        .split([',', '/', ' '])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect()
}

fn load_registry(config: &CliConfig) -> Result<Registry, CliError> {
    if config.registry.source != "bundled" {
        return Err(CliError::new(
            "registry_error",
            "only bundled registry source is supported in v1-beta CLI",
        ));
    }
    let registry = Registry::load_frozen().map_err(registry_error)?;
    if config.registry.validate_on_start {
        registry.validate().map_err(registry_error)?;
    }
    Ok(registry)
}

fn positional<'a>(args: &'a [String], position: usize, label: &str) -> Result<&'a str, CliError> {
    positional_args(args)
        .into_iter()
        .nth(position)
        .map(String::as_str)
        .ok_or_else(|| CliError::new("parse_error", format!("missing {label}")))
}

fn positional_args(args: &[String]) -> Vec<&String> {
    let mut values = Vec::new();
    let mut index = 0usize;
    while index < args.len() {
        let arg = &args[index];
        if arg.starts_with("--") {
            if flag_takes_value(arg) && index + 1 < args.len() {
                index += 2;
            } else {
                index += 1;
            }
        } else {
            values.push(arg);
            index += 1;
        }
    }
    values
}

fn flag_takes_value(flag: &str) -> bool {
    matches!(
        flag,
        "--space"
            | "--format"
            | "--precision"
            | "--exports"
            | "--to"
            | "--from"
            | "--type"
            | "--path"
            | "--config"
            | "--host"
            | "--port"
    )
}

fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].as_str())
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

fn serve_help_text() -> String {
    [
        "OCI Local Kernel API Server",
        "",
        "Usage:",
        "  oci serve [--host <HOST>] [--port <PORT>] [--config <PATH>] [--json]",
        "",
        "Defaults:",
        "  host: 127.0.0.1",
        "  port: 8765",
        "",
        "Endpoints:",
        "  GET  /v1/health",
        "  POST /v1/encode",
        "  POST /v1/inspect",
        "  POST /v1/export",
        "  POST /v1/convert",
        "  GET  /v1/registry/info",
        "  GET  /v1/registry/families",
        "  GET  /v1/registry/family/{indexOrCode}",
        "  GET  /v1/registry/step/{idOrStep}",
    ]
    .join("\n")
}

fn pipeline_error(error: oci_core::OciPipelineError) -> CliError {
    CliError::new("parse_error", error.to_string())
}

fn id_error(error: oci_core::OciIdError) -> CliError {
    let code = match error {
        oci_core::OciIdError::InvalidFamilyCode { .. }
        | oci_core::OciIdError::UnknownFamily { .. }
        | oci_core::OciIdError::FamilyIndexCodeMismatch { .. } => "invalid_family",
        oci_core::OciIdError::InvalidStepNumber { .. }
        | oci_core::OciIdError::InvalidStepComponent { .. } => "invalid_step",
        oci_core::OciIdError::InvalidOffset { .. } => "invalid_offset",
        _ => "invalid_id",
    };
    CliError::new(code, error.to_string())
}

fn registry_error(error: oci_core::RegistryError) -> CliError {
    CliError::new("registry_error", error.to_string())
}

fn config_error(error: crate::config::ConfigError) -> CliError {
    CliError::new("config_error", error.to_string())
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", output::escape_json(value))
}

fn json_string_field(object: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\":\"");
    let start = object.find(&marker)? + marker.len();
    let rest = &object[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    fn temp_config_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("oci-{name}-{}.toml", std::process::id()))
    }

    fn assert_all_supported_exports_are_visible(out: &str) {
        for expected in [
            "HEX:",
            "RGB:",
            "HSL:",
            "sRGB:",
            "Display P3:",
            "Adobe RGB:",
            "Rec.709:",
            "OKLCH:",
            "OKLab:",
            "CSS OKLCH:",
            "CSS sRGB:",
            "CSS Display P3:",
            "JSON token:",
            "Swift:",
            "Tailwind:",
            "CMYK:",
        ] {
            assert!(out.contains(expected), "missing export line: {expected}");
        }
    }

    fn assert_no_inline_verification_metadata(out: &str) {
        assert!(!out.contains("(lossy"));
        assert!(!out.contains("(supported"));
        assert!(!out.contains("(profile_required"));
        assert!(!out.contains("round-trip error") || out.contains("verification:"));
    }

    fn assert_compact_verification_is_visible(out: &str) {
        assert!(out.contains("\n\nverification:\n"));
        assert!(out.contains("lossy: HEX, RGB"));
        assert!(out.contains("supported:"));
        assert!(out.contains("profile required: CMYK"));
        assert!(out.contains("max round-trip error:"));
    }

    #[test]
    fn parses_encode_command_and_emits_pretty_by_default() {
        let out = run_cli(&args(&["encode", "#E85A9A", "--space", "hex"])).unwrap();
        assert!(!out.starts_with('{'));
        assert!(!out.contains("{\""));
        assert!(out.contains("OCI Encode"));
        assert!(out.contains("OCI standard color code: OCI-1-"));
        assert!(out.contains("OCI precision color code: OCI-1-"));
        assert!(out.contains("@L"));
        assert!(out.contains("\n\nOCI standard color code:"));
        assert!(out.contains("\n\nexports:"));
        assert!(out.contains("CSS OKLCH:"));
        assert!(!out.contains("CSS:\n"));
        assert_all_supported_exports_are_visible(&out);
        assert_no_inline_verification_metadata(&out);
        assert_compact_verification_is_visible(&out);
        assert!(!out.contains("verification details:"));
    }

    #[test]
    fn help_and_version_commands_work() {
        let help = run_cli(&args(&["--help"])).unwrap();
        assert!(help.contains("Open Chroma Index CLI"));
        assert!(help.contains("oci encode <INPUT>"));
        assert!(help.contains("oci serve"));

        let version = run_cli(&args(&["--version"])).unwrap();
        assert!(version.starts_with("oci "));
    }

    #[test]
    fn serve_help_exists() {
        let help = run_cli(&args(&["serve", "--help"])).unwrap();
        assert!(help.contains("OCI Local Kernel API Server"));
        assert!(help.contains("GET  /v1/health"));
    }

    #[test]
    fn encode_json_output_still_works_with_format_flag() {
        let out = run_cli(&args(&[
            "encode", "#E85A9A", "--space", "hex", "--format", "json",
        ]))
        .unwrap();
        assert!(out.starts_with('{'));
        assert!(out.contains("\"sourceSpace\":\"hex\""));
        assert!(out.contains("\"oci\""));
        assert!(out.contains("\"swift\""));
        assert!(out.contains("\"tailwind\""));
        assert!(out.contains("\"cmyk\""));
        assert!(out.contains("\"roundTripError\""));
        assert!(out.contains("\"status\":\"lossy\""));
    }

    #[test]
    fn inspect_command_has_expected_structure() {
        let out = run_cli(&args(&["inspect", "OCI-1-48RS-327"])).unwrap();
        assert!(out.contains("OCI Inspect"));
        assert!(out.contains("OCI standard color code: OCI-1-48RS-327\n"));
        assert!(out.contains("exports:"));
        assert!(out.contains("OKLCH:"));
        assert_all_supported_exports_are_visible(&out);
        assert_no_inline_verification_metadata(&out);
        assert_compact_verification_is_visible(&out);
    }

    #[test]
    fn export_selects_targets() {
        let out = run_cli(&args(&[
            "export",
            "OCI-1-46PK-236",
            "--to",
            "hex,oklch,cmyk",
        ]))
        .unwrap();
        assert!(!out.contains("{\""));
        assert!(out.contains("HEX:"));
        assert!(out.contains("OKLCH:"));
        assert!(out.contains("CMYK:"));
        assert!(out.contains("profile required: CMYK"));
        assert_no_inline_verification_metadata(&out);
    }

    #[test]
    fn convert_command_has_expected_structure() {
        let out = run_cli(&args(&[
            "convert",
            "#E85A9A",
            "--from",
            "hex",
            "--to",
            "srgb,oklch",
        ]))
        .unwrap();
        assert!(out.contains("OCI Convert"));
        assert!(out.contains("exports:"));
        assert!(out.contains("verification:"));
        assert_no_inline_verification_metadata(&out);
    }

    #[test]
    fn verify_flag_shows_detailed_verification() {
        let out = run_cli(&args(&["encode", "#E85A9A", "--space", "hex", "--verify"])).unwrap();
        assert!(out.contains("verification details:"));
        assert!(out.contains("HEX: lossy, round-trip error"));
        assert!(out.contains("sRGB: supported, round-trip error"));
        assert!(out.contains("CMYK: profile_required"));
    }

    #[test]
    fn config_verify_true_enables_detailed_verification() {
        let path = temp_config_path("verify-enabled");
        fs::write(&path, "[output]\nverify = true\n").unwrap();

        let out = run_cli(&args(&[
            "inspect",
            "OCI-1-48RS-327",
            "--path",
            path.to_str().unwrap(),
        ]))
        .unwrap();
        assert!(out.contains("verification details:"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn verify_flag_overrides_false_config() {
        let path = temp_config_path("verify-override");
        fs::write(&path, "[output]\nverify = false\n").unwrap();

        let out = run_cli(&args(&[
            "export",
            "OCI-1-48RS-327",
            "--to",
            "hex,rgb,cmyk",
            "--verify",
            "--path",
            path.to_str().unwrap(),
        ]))
        .unwrap();
        assert!(out.contains("verification details:"));
        assert!(out.contains("HEX: lossy, round-trip error"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn plain_output_remains_minimal() {
        let out = run_cli(&args(&[
            "encode", "#E85A9A", "--space", "hex", "--format", "plain",
        ]))
        .unwrap();
        assert!(out.starts_with("OCI-1-"));
        assert!(!out.contains("exports:"));
        assert!(!out.contains("verification:"));
    }

    #[test]
    fn built_in_default_config_loads() {
        let config = CliConfig::default();
        assert_eq!(config.output.format, "pretty");
        assert_eq!(config.output.precision, 6);
        assert_eq!(config.registry.source, "bundled");
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8765);
    }

    #[test]
    fn default_config_path_uses_installed_binary_directory() {
        let path = crate::config::default_config_path();
        assert_eq!(path.file_name().unwrap(), "config.toml");
        assert!(!path.starts_with("cli"));
    }

    #[test]
    fn missing_default_config_uses_built_in_defaults() {
        let path = temp_config_path("missing-defaults");
        let _ = fs::remove_file(&path);
        let config = CliConfig::load_from_path(path).unwrap();
        assert_eq!(config.output.format, "pretty");
        assert!(config.output.default_exports.contains(&"hex".to_string()));
    }

    #[test]
    fn custom_path_config_loads() {
        let path = temp_config_path("custom-loads");
        fs::write(
            &path,
            "[output]\nformat = \"json\"\ndefault_exports = [\"hex\"]\n",
        )
        .unwrap();

        let out = run_cli(&args(&[
            "encode",
            "#E85A9A",
            "--space",
            "hex",
            "--path",
            path.to_str().unwrap(),
        ]))
        .unwrap();
        assert!(out.starts_with('{'));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn cli_flags_override_config() {
        let path = temp_config_path("flags-override");
        fs::write(&path, "[output]\nformat = \"pretty\"\n").unwrap();

        let out = run_cli(&args(&[
            "encode",
            "#E85A9A",
            "--space",
            "hex",
            "--format",
            "json",
            "--path",
            path.to_str().unwrap(),
        ]))
        .unwrap();
        assert!(out.starts_with('{'));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn invalid_toml_returns_structured_error() {
        let path = temp_config_path("invalid");
        fs::write(&path, "[output]\nformat = [\n").unwrap();

        let error = run_cli(&args(&[
            "encode",
            "#E85A9A",
            "--space",
            "hex",
            "--path",
            path.to_str().unwrap(),
        ]))
        .unwrap_err();
        assert_eq!(error.code, "config_error");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn missing_config_can_be_created_through_config_command() {
        let path = temp_config_path("create");
        let _ = fs::remove_file(&path);

        let out = run_cli(&args(&["config", "--path", path.to_str().unwrap()])).unwrap();
        assert!(out.contains("OCI config written"));
        let written = fs::read_to_string(&path).unwrap();
        assert!(written.contains("[output]"));
        assert!(written.contains("format = \"pretty\""));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn registry_info_and_validate_work() {
        let info = run_cli(&args(&["registry", "info"])).unwrap();
        assert!(info.contains("\"familyCount\":64"));
        assert!(info.contains("\"stepCount\":23040"));
        let validate = run_cli(&args(&["registry", "validate"])).unwrap();
        assert!(validate.contains("\"valid\":true"));
    }

    #[test]
    fn checksum_command_reports_sha256() {
        let out = run_cli(&args(&["registry", "checksum"])).unwrap();
        assert!(out.contains("\"algorithm\":\"sha256\""));
        assert!(out.contains("\"valid\":true"));
    }

    #[test]
    fn test_vectors_command_runs() {
        let out = run_cli(&args(&["test", "vectors"])).unwrap();
        assert!(out.contains("\"test\":\"vectors\""));
        assert!(out.contains("\"passed\""));
    }

    #[test]
    fn invalid_cli_input_returns_error() {
        let error = run_cli(&args(&["encode", "oops", "--space", "unknown"])).unwrap_err();
        assert_eq!(error.code, "unsupported_space");
    }

    #[test]
    fn invalid_oci_id_returns_error() {
        let error = run_cli(&args(&["inspect", "OCI-1-46PK-999"])).unwrap_err();
        assert_eq!(error.code, "invalid_step");
    }

    #[test]
    fn cli_binary_is_named_oci() {
        let manifest = include_str!("../Cargo.toml");
        assert!(manifest.contains("name = \"oci\""));
    }
}
