use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliConfig {
    pub output: OutputConfig,
    pub encode: EncodeConfig,
    pub inspect: InspectConfig,
    pub registry: RegistryConfig,
    pub color: ColorConfig,
    pub server: ServerConfig,
    pub update: UpdateConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputConfig {
    pub format: String,
    pub precision: usize,
    pub show_support: bool,
    pub show_warnings: bool,
    pub show_exports: bool,
    pub verify: bool,
    pub default_exports: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodeConfig {
    pub include_offset: bool,
    pub prefer_short_code: bool,
    pub include_full_code: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectConfig {
    pub exports: String,
    pub default_export_list: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryConfig {
    pub version: String,
    pub source: String,
    pub path: String,
    pub validate_on_start: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorConfig {
    pub default_input_space: String,
    pub default_targets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub warn_non_localhost: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateConfig {
    pub check: bool,
    pub notices_shown: usize,
    pub last_seen_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    pub message: String,
}

impl ConfigError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ConfigError {}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            output: OutputConfig {
                format: "pretty".to_string(),
                precision: 6,
                show_support: true,
                show_warnings: true,
                show_exports: true,
                verify: false,
                default_exports: all_export_targets(),
            },
            encode: EncodeConfig {
                include_offset: true,
                prefer_short_code: true,
                include_full_code: false,
            },
            inspect: InspectConfig {
                exports: "summary".to_string(),
                default_export_list: all_export_targets(),
            },
            registry: RegistryConfig {
                version: "v1".to_string(),
                source: "bundled".to_string(),
                path: String::new(),
                validate_on_start: false,
            },
            color: ColorConfig {
                default_input_space: "hex".to_string(),
                default_targets: vec![
                    "hex".to_string(),
                    "oklch".to_string(),
                    "display-p3".to_string(),
                ],
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8765,
                warn_non_localhost: true,
            },
            update: UpdateConfig {
                check: true,
                notices_shown: 0,
                last_seen_version: String::new(),
            },
        }
    }
}

pub fn all_export_targets() -> Vec<String> {
    [
        "hex",
        "rgb",
        "hsl",
        "srgb",
        "display-p3",
        "adobe-rgb",
        "rec709",
        "oklch",
        "oklab",
        "css",
        "json-token",
        "swift",
        "tailwind",
        "cmyk",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

impl CliConfig {
    pub fn load_from_args(args: &[String]) -> Result<Self, ConfigError> {
        Self::load_from_path(config_path_from_args(args))
    }

    pub fn load_from_path(path: PathBuf) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path).map_err(|error| {
            ConfigError::new(format!("failed to read {}: {error}", path.display()))
        })?;
        Self::from_toml_str(&content).map_err(|error| {
            ConfigError::new(format!("invalid TOML in {}: {error}", path.display()))
        })
    }

    pub fn from_toml_str(content: &str) -> Result<Self, ConfigError> {
        let mut config = Self::default();
        let mut section = String::new();

        for (line_index, raw_line) in content.lines().enumerate() {
            let line_number = line_index + 1;
            let line = strip_comment(raw_line).trim().to_string();
            if line.is_empty() {
                continue;
            }

            if line.starts_with('[') {
                if !line.ends_with(']') || line.len() <= 2 {
                    return Err(ConfigError::new(format!(
                        "line {line_number}: invalid section header"
                    )));
                }
                section = line[1..line.len() - 1].trim().to_string();
                match section.as_str() {
                    "output" | "encode" | "inspect" | "registry" | "color" | "server"
                    | "update" => {}
                    _ => {
                        return Err(ConfigError::new(format!(
                            "line {line_number}: unknown section [{section}]"
                        )));
                    }
                }
                continue;
            }

            let Some((key, value)) = line.split_once('=') else {
                return Err(ConfigError::new(format!(
                    "line {line_number}: expected key = value"
                )));
            };
            config.set_value(&section, key.trim(), value.trim(), line_number)?;
        }

        Ok(config)
    }

    pub fn to_toml_string(&self) -> String {
        format!(
            concat!(
                "[output]\n",
                "format = \"{}\"\n",
                "precision = {}\n",
                "show_support = {}\n",
                "show_warnings = {}\n",
                "show_exports = {}\n",
                "verify = {}\n",
                "default_exports = {}\n\n",
                "[encode]\n",
                "include_offset = {}\n",
                "prefer_short_code = {}\n",
                "include_full_code = {}\n\n",
                "[inspect]\n",
                "exports = \"{}\"\n",
                "default_export_list = {}\n\n",
                "[registry]\n",
                "version = \"{}\"\n",
                "source = \"{}\"\n",
                "path = \"{}\"\n",
                "validate_on_start = {}\n\n",
                "[color]\n",
                "default_input_space = \"{}\"\n",
                "default_targets = {}\n\n",
                "[server]\n",
                "host = \"{}\"\n",
                "port = {}\n",
                "warn_non_localhost = {}\n\n",
                "[update]\n",
                "check = {}\n",
                "notices_shown = {}\n",
                "last_seen_version = \"{}\"\n"
            ),
            escape_toml_string(&self.output.format),
            self.output.precision,
            self.output.show_support,
            self.output.show_warnings,
            self.output.show_exports,
            self.output.verify,
            toml_array(&self.output.default_exports),
            self.encode.include_offset,
            self.encode.prefer_short_code,
            self.encode.include_full_code,
            escape_toml_string(&self.inspect.exports),
            toml_array(&self.inspect.default_export_list),
            escape_toml_string(&self.registry.version),
            escape_toml_string(&self.registry.source),
            escape_toml_string(&self.registry.path),
            self.registry.validate_on_start,
            escape_toml_string(&self.color.default_input_space),
            toml_array(&self.color.default_targets),
            escape_toml_string(&self.server.host),
            self.server.port,
            self.server.warn_non_localhost,
            self.update.check,
            self.update.notices_shown,
            escape_toml_string(&self.update.last_seen_version)
        )
    }

    pub fn write_to_path(&self, path: &Path) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent).map_err(|error| {
                ConfigError::new(format!("failed to create {}: {error}", parent.display()))
            })?;
        }
        fs::write(path, self.to_toml_string()).map_err(|error| {
            ConfigError::new(format!("failed to write {}: {error}", path.display()))
        })
    }

    fn set_value(
        &mut self,
        section: &str,
        key: &str,
        value: &str,
        line_number: usize,
    ) -> Result<(), ConfigError> {
        match (section, key) {
            ("output", "format") => self.output.format = parse_string(value, line_number)?,
            ("output", "precision") => self.output.precision = parse_usize(value, line_number)?,
            ("output", "show_support") => {
                self.output.show_support = parse_bool(value, line_number)?
            }
            ("output", "show_warnings") => {
                self.output.show_warnings = parse_bool(value, line_number)?
            }
            ("output", "show_exports") => {
                self.output.show_exports = parse_bool(value, line_number)?
            }
            ("output", "verify") => self.output.verify = parse_bool(value, line_number)?,
            ("output", "default_exports") => {
                self.output.default_exports = parse_string_array(value, line_number)?
            }
            ("encode", "include_offset") => {
                self.encode.include_offset = parse_bool(value, line_number)?
            }
            ("encode", "prefer_short_code") => {
                self.encode.prefer_short_code = parse_bool(value, line_number)?
            }
            ("encode", "include_full_code") => {
                self.encode.include_full_code = parse_bool(value, line_number)?
            }
            ("inspect", "exports") => self.inspect.exports = parse_string(value, line_number)?,
            ("inspect", "default_export_list") => {
                self.inspect.default_export_list = parse_string_array(value, line_number)?
            }
            ("registry", "version") => self.registry.version = parse_string(value, line_number)?,
            ("registry", "source") => self.registry.source = parse_string(value, line_number)?,
            ("registry", "path") => self.registry.path = parse_string(value, line_number)?,
            ("registry", "validate_on_start") => {
                self.registry.validate_on_start = parse_bool(value, line_number)?
            }
            ("color", "default_input_space") => {
                self.color.default_input_space = parse_string(value, line_number)?
            }
            ("color", "default_targets") => {
                self.color.default_targets = parse_string_array(value, line_number)?
            }
            ("server", "host") => self.server.host = parse_string(value, line_number)?,
            ("server", "port") => {
                let port = parse_usize(value, line_number)?;
                if port > u16::MAX as usize {
                    return Err(ConfigError::new(format!(
                        "line {line_number}: server.port must be <= {}",
                        u16::MAX
                    )));
                }
                self.server.port = port as u16;
            }
            ("server", "warn_non_localhost") => {
                self.server.warn_non_localhost = parse_bool(value, line_number)?
            }
            ("update", "check") => self.update.check = parse_bool(value, line_number)?,
            ("update", "notices_shown") => {
                self.update.notices_shown = parse_usize(value, line_number)?
            }
            ("update", "last_seen_version") => {
                self.update.last_seen_version = parse_string(value, line_number)?
            }
            ("", _) => {
                return Err(ConfigError::new(format!(
                    "line {line_number}: key must be inside a section"
                )));
            }
            _ => {
                return Err(ConfigError::new(format!(
                    "line {line_number}: unknown key {section}.{key}"
                )));
            }
        }
        Ok(())
    }
}

pub fn config_path_from_args(args: &[String]) -> PathBuf {
    args.windows(2)
        .find(|pair| pair[0] == "--path" || pair[0] == "--config")
        .map_or_else(default_config_path, |pair| PathBuf::from(&pair[1]))
}

pub fn default_config_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .map_or_else(
            || PathBuf::from(DEFAULT_CONFIG_FILE_NAME),
            |directory| directory.join(DEFAULT_CONFIG_FILE_NAME),
        )
}

fn parse_string(value: &str, line_number: usize) -> Result<String, ConfigError> {
    if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
        return Err(ConfigError::new(format!(
            "line {line_number}: expected quoted string"
        )));
    }
    Ok(unescape_toml_string(&value[1..value.len() - 1]))
}

fn parse_bool(value: &str, line_number: usize) -> Result<bool, ConfigError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(ConfigError::new(format!(
            "line {line_number}: expected boolean"
        ))),
    }
}

fn parse_usize(value: &str, line_number: usize) -> Result<usize, ConfigError> {
    value
        .parse::<usize>()
        .map_err(|_| ConfigError::new(format!("line {line_number}: expected unsigned integer")))
}

fn parse_string_array(value: &str, line_number: usize) -> Result<Vec<String>, ConfigError> {
    if value.len() < 2 || !value.starts_with('[') || !value.ends_with(']') {
        return Err(ConfigError::new(format!(
            "line {line_number}: expected string array"
        )));
    }

    let inner = value[1..value.len() - 1].trim();
    if inner.is_empty() {
        return Ok(Vec::new());
    }

    inner
        .split(',')
        .map(str::trim)
        .map(|item| parse_string(item, line_number))
        .collect()
}

fn strip_comment(line: &str) -> String {
    let mut in_string = false;
    let mut escaped = false;
    let mut out = String::new();

    for ch in line.chars() {
        if escaped {
            out.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => {
                out.push(ch);
                escaped = true;
            }
            '"' => {
                in_string = !in_string;
                out.push(ch);
            }
            '#' if !in_string => break,
            _ => out.push(ch),
        }
    }

    out
}

fn toml_array(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| format!("\"{}\"", escape_toml_string(value)))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn unescape_toml_string(value: &str) -> String {
    let mut out = String::new();
    let mut escaped = false;
    for ch in value.chars() {
        if escaped {
            out.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else {
            out.push(ch);
        }
    }
    out
}
