use crate::commands::{self, CliError};
use crate::config::CliConfig;
use crate::output;
use tiny_http::{Header, Request, Response, Server, StatusCode};

pub const API_VERSION: &str = "0.2.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerOptions {
    pub host: String,
    pub port: u16,
    pub json_startup: bool,
    pub warn_non_localhost: bool,
}

impl ServerOptions {
    pub fn from_args(args: &[String], config: &CliConfig) -> Result<Self, CliError> {
        let host = flag_value(args, "--host")
            .unwrap_or(&config.server.host)
            .to_string();
        let port = flag_value(args, "--port").map_or(Ok(config.server.port), |value| {
            value
                .parse::<u16>()
                .map_err(|_| CliError::new("parse_error", format!("invalid port: {value}")))
        })?;

        Ok(Self {
            host,
            port,
            json_startup: has_flag(args, "--json"),
            warn_non_localhost: config.server.warn_non_localhost,
        })
    }

    pub fn url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

pub fn serve(options: ServerOptions, config: CliConfig) -> Result<(), CliError> {
    let address = format!("{}:{}", options.host, options.port);
    let server = Server::http(&address).map_err(|error| {
        CliError::new(
            "server_error",
            format!("failed to bind Local Kernel API server at {address}: {error}"),
        )
    })?;

    print_startup(&options);

    loop {
        let request = server
            .recv()
            .map_err(|error| CliError::new("server_error", error.to_string()))?;
        respond(request, &config)?;
    }
}

fn print_startup(options: &ServerOptions) {
    if options.json_startup {
        println!(
            "{{\"ok\":true,\"data\":{{\"service\":\"oci-local-api\",\"version\":\"{}\",\"url\":\"{}\",\"host\":\"{}\",\"port\":{}}}}}",
            API_VERSION,
            output::escape_json(&options.url()),
            output::escape_json(&options.host),
            options.port
        );
    } else {
        println!("OCI Local Kernel API listening on {}", options.url());
    }

    if options.warn_non_localhost && !is_localhost(&options.host) {
        eprintln!(
            "warning: OCI Local Kernel API is bound to {}; prefer 127.0.0.1 unless remote access is intentional",
            options.host
        );
    }
}

fn is_localhost(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "::1" | "localhost")
}

fn respond(mut request: Request, config: &CliConfig) -> Result<(), CliError> {
    let method = request.method().to_string();
    let url = request.url().to_string();
    let mut body = String::new();
    request
        .as_reader()
        .read_to_string(&mut body)
        .map_err(|error| CliError::new("server_error", error.to_string()))?;

    let (status, response_body) = handle_api_request(&method, &url, &body, config);
    let header = Header::from_bytes(
        &b"Content-Type"[..],
        &b"application/json; charset=utf-8"[..],
    )
    .map_err(|_| CliError::new("server_error", "failed to create response header"))?;
    request
        .respond(
            Response::from_string(response_body)
                .with_status_code(StatusCode(status))
                .with_header(header),
        )
        .map_err(|error| CliError::new("server_error", error.to_string()))
}

#[cfg(test)]
fn serve_one_request(server: &Server, config: &CliConfig) -> Result<(), CliError> {
    let request = server
        .recv_timeout(std::time::Duration::from_secs(5))
        .map_err(|error| CliError::new("server_error", error.to_string()))?
        .ok_or_else(|| CliError::new("server_error", "timed out waiting for request"))?;
    respond(request, config)
}

pub(crate) fn handle_api_request(
    method: &str,
    path: &str,
    body: &str,
    config: &CliConfig,
) -> (u16, String) {
    let path = path.split('?').next().unwrap_or(path);

    match (method, path) {
        ("GET", "/v1/health") => success(health_json()),
        ("POST", "/v1/encode") => encode_endpoint(body, config),
        ("POST", "/v1/inspect") => inspect_endpoint(body, config),
        ("POST", "/v1/export") => export_endpoint(body, config),
        ("POST", "/v1/convert") => convert_endpoint(body, config),
        ("GET", "/v1/registry/info") => command_endpoint(&["registry", "info"], config),
        ("GET", "/v1/registry/families") => command_endpoint(&["registry", "families"], config),
        _ if method == "GET" && path.starts_with("/v1/registry/family/") => {
            let key = path.trim_start_matches("/v1/registry/family/");
            if key.is_empty() {
                error(400, "invalid_input", "missing family index or code")
            } else {
                command_endpoint(&["registry", "family", key], config)
            }
        }
        _ if method == "GET" && path.starts_with("/v1/registry/step/") => {
            let key = path.trim_start_matches("/v1/registry/step/");
            if key.is_empty() {
                error(400, "invalid_input", "missing OCI ID or step ID")
            } else {
                command_endpoint(&["registry", "step", key], config)
            }
        }
        _ if known_path(path) => error(
            405,
            "method_not_allowed",
            format!("method {method} is not allowed for {path}"),
        ),
        _ => error(404, "not_found", format!("unknown endpoint: {path}")),
    }
}

fn health_json() -> String {
    format!(
        "{{\"status\":\"ok\",\"service\":\"oci-local-api\",\"version\":\"{}\"}}",
        API_VERSION
    )
}

fn encode_endpoint(body: &str, config: &CliConfig) -> (u16, String) {
    let input = match required_string_field(body, "input") {
        Ok(value) => value,
        Err(error) => return api_error(error),
    };
    let space = json_string_field(body, "space").unwrap_or_else(|| "hex".to_string());
    let precision = json_usize_field(body, "precision").unwrap_or(config.output.precision);
    command_endpoint_owned(
        vec![
            "encode".to_string(),
            input,
            "--space".to_string(),
            space,
            "--format".to_string(),
            "json".to_string(),
            "--precision".to_string(),
            precision.to_string(),
        ],
        config,
    )
}

fn inspect_endpoint(body: &str, config: &CliConfig) -> (u16, String) {
    let id = match required_string_field(body, "id") {
        Ok(value) => value,
        Err(error) => return api_error(error),
    };
    let exports = json_string_field(body, "exports").unwrap_or_else(|| "summary".to_string());
    command_endpoint_owned(
        vec![
            "inspect".to_string(),
            id,
            "--format".to_string(),
            "json".to_string(),
            "--exports".to_string(),
            exports,
        ],
        config,
    )
}

fn export_endpoint(body: &str, config: &CliConfig) -> (u16, String) {
    let id = match required_string_field(body, "id") {
        Ok(value) => value,
        Err(error) => return api_error(error),
    };
    let targets = json_string_array_field(body, "targets")
        .filter(|targets| !targets.is_empty())
        .unwrap_or_else(|| config.output.default_exports.clone());
    command_endpoint_owned(
        vec![
            "export".to_string(),
            id,
            "--to".to_string(),
            targets.join(","),
            "--format".to_string(),
            "json".to_string(),
        ],
        config,
    )
}

fn convert_endpoint(body: &str, config: &CliConfig) -> (u16, String) {
    let input = match required_string_field(body, "input") {
        Ok(value) => value,
        Err(error) => return api_error(error),
    };
    let from =
        json_string_field(body, "from").unwrap_or_else(|| config.color.default_input_space.clone());
    let targets = json_string_array_field(body, "to")
        .filter(|targets| !targets.is_empty())
        .unwrap_or_else(|| config.color.default_targets.clone());
    command_endpoint_owned(
        vec![
            "convert".to_string(),
            input,
            "--from".to_string(),
            from,
            "--to".to_string(),
            targets.join(","),
            "--format".to_string(),
            "json".to_string(),
        ],
        config,
    )
}

fn command_endpoint(values: &[&str], config: &CliConfig) -> (u16, String) {
    command_endpoint_owned(
        values.iter().map(|value| (*value).to_string()).collect(),
        config,
    )
}

fn command_endpoint_owned(values: Vec<String>, config: &CliConfig) -> (u16, String) {
    match commands::run_cli_with_config(&values, config) {
        Ok(data) => success(data),
        Err(error) => {
            let code = api_code(&error.code);
            self::error(status_for_code(code), code, error.message)
        }
    }
}

fn success(data: String) -> (u16, String) {
    (200, format!("{{\"ok\":true,\"data\":{data}}}"))
}

fn error(status: u16, code: impl AsRef<str>, message: impl AsRef<str>) -> (u16, String) {
    (
        status,
        format!(
            "{{\"ok\":false,\"error\":{{\"code\":\"{}\",\"message\":\"{}\"}}}}",
            output::escape_json(code.as_ref()),
            output::escape_json(message.as_ref())
        ),
    )
}

fn api_error(error: ApiError) -> (u16, String) {
    self::error(status_for_code(&error.code), error.code, error.message)
}

fn api_code(code: &str) -> &str {
    match code {
        "parse_error" => "invalid_input",
        other => other,
    }
}

fn status_for_code(code: &str) -> u16 {
    match code {
        "not_found" => 404,
        "method_not_allowed" => 405,
        "registry_error" | "server_error" => 500,
        _ => 400,
    }
}

fn known_path(path: &str) -> bool {
    matches!(
        path,
        "/v1/health"
            | "/v1/encode"
            | "/v1/inspect"
            | "/v1/export"
            | "/v1/convert"
            | "/v1/registry/info"
            | "/v1/registry/families"
    ) || path.starts_with("/v1/registry/family/")
        || path.starts_with("/v1/registry/step/")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ApiError {
    code: String,
    message: String,
}

impl ApiError {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

fn required_string_field(body: &str, key: &str) -> Result<String, ApiError> {
    json_string_field(body, key)
        .ok_or_else(|| ApiError::new("invalid_input", format!("missing JSON string field: {key}")))
}

fn json_string_field(body: &str, key: &str) -> Option<String> {
    let value = json_value_after_key(body, key)?;
    parse_json_string(value).map(|(value, _)| value)
}

fn json_usize_field(body: &str, key: &str) -> Option<usize> {
    let value = json_value_after_key(body, key)?;
    let digits = value
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

fn json_string_array_field(body: &str, key: &str) -> Option<Vec<String>> {
    let mut rest = json_value_after_key(body, key)?.trim_start();
    if !rest.starts_with('[') {
        return None;
    }
    rest = &rest[1..];
    let mut values = Vec::new();

    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            return Some(values);
        }
        let (value, consumed) = parse_json_string(rest)?;
        values.push(value);
        rest = rest[consumed..].trim_start();
        if rest.starts_with(',') {
            rest = &rest[1..];
        } else if rest.starts_with(']') {
            return Some(values);
        } else {
            return None;
        }
    }
}

fn json_value_after_key<'a>(body: &'a str, key: &str) -> Option<&'a str> {
    let marker = format!("\"{key}\"");
    let start = body.find(&marker)? + marker.len();
    let rest = body[start..].trim_start();
    let rest = rest.strip_prefix(':')?.trim_start();
    Some(rest)
}

fn parse_json_string(input: &str) -> Option<(String, usize)> {
    let mut chars = input.char_indices();
    let (_, first) = chars.next()?;
    if first != '"' {
        return None;
    }

    let mut escaped = false;
    let mut value = String::new();
    for (index, ch) in chars {
        if escaped {
            match ch {
                '"' | '\\' | '/' => value.push(ch),
                'n' => value.push('\n'),
                'r' => value.push('\r'),
                't' => value.push('\t'),
                _ => value.push(ch),
            }
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Some((value, index + ch.len_utf8()));
        } else {
            value.push(ch);
        }
    }
    None
}

fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].as_str())
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};

    fn config() -> CliConfig {
        CliConfig::default()
    }

    #[test]
    fn default_server_options_bind_to_localhost() {
        let options = ServerOptions::from_args(&[], &config()).unwrap();
        assert_eq!(options.host, "127.0.0.1");
        assert_eq!(options.port, 8765);
    }

    #[test]
    fn server_flags_override_config_defaults() {
        let mut config = config();
        config.server.host = "127.0.0.1".to_string();
        config.server.port = 8765;
        let options = ServerOptions::from_args(
            &[
                "--host".to_string(),
                "localhost".to_string(),
                "--port".to_string(),
                "9000".to_string(),
            ],
            &config,
        )
        .unwrap();
        assert_eq!(options.host, "localhost");
        assert_eq!(options.port, 9000);
    }

    #[test]
    fn health_endpoint_returns_expected_envelope() {
        let (_, body) = handle_api_request("GET", "/v1/health", "", &config());
        assert!(body.contains("\"ok\":true"));
        assert!(body.contains("\"service\":\"oci-local-api\""));
        assert!(body.contains("\"version\":\"0.2.0\""));
    }

    #[test]
    fn encode_endpoint_reuses_cli_json_structure() {
        let (_, body) = handle_api_request(
            "POST",
            "/v1/encode",
            "{\"input\":\"#E85A9A\",\"space\":\"hex\",\"precision\":6}",
            &config(),
        );
        assert!(body.contains("\"ok\":true"));
        assert!(body.contains("\"sourceSpace\":\"hex\""));
        assert!(body.contains("\"precisionShort\":\"OCI-1-"));
    }

    #[test]
    fn inspect_endpoint_returns_json_structure() {
        let (_, body) = handle_api_request(
            "POST",
            "/v1/inspect",
            "{\"id\":\"OCI-1-48RS-327\",\"exports\":\"summary\"}",
            &config(),
        );
        assert!(body.contains("\"ok\":true"));
        assert!(body.contains("\"canonical\""));
        assert!(body.contains("\"baseStep\""));
    }

    #[test]
    fn export_endpoint_selects_targets() {
        let (_, body) = handle_api_request(
            "POST",
            "/v1/export",
            "{\"id\":\"OCI-1-48RS-327\",\"targets\":[\"hex\",\"oklch\",\"css\"]}",
            &config(),
        );
        assert!(body.contains("\"ok\":true"));
        assert!(body.contains("\"hex\""));
        assert!(body.contains("\"oklch\""));
        assert!(body.contains("\"css\""));
        assert!(!body.contains("\"adobeRgb1998\""));
    }

    #[test]
    fn convert_endpoint_returns_exports() {
        let (_, body) = handle_api_request(
            "POST",
            "/v1/convert",
            "{\"input\":\"232,90,154\",\"from\":\"rgb\",\"to\":[\"srgb\",\"display-p3\",\"oklch\"]}",
            &config(),
        );
        assert!(body.contains("\"ok\":true"));
        assert!(body.contains("\"displayP3\""));
        assert!(body.contains("\"oklch\""));
    }

    #[test]
    fn registry_info_endpoint_works() {
        let (_, body) = handle_api_request("GET", "/v1/registry/info", "", &config());
        assert!(body.contains("\"ok\":true"));
        assert!(body.contains("\"familyCount\":64"));
        assert!(body.contains("\"stepCount\":23040"));
    }

    #[test]
    fn registry_families_endpoint_works() {
        let (_, body) = handle_api_request("GET", "/v1/registry/families", "", &config());
        assert!(body.contains("\"ok\":true"));
        assert!(body.contains("\"families\""));
        assert!(body.contains("\"code\":\"RS\""));
    }

    #[test]
    fn registry_family_endpoint_accepts_code() {
        let (_, body) = handle_api_request("GET", "/v1/registry/family/RS", "", &config());
        assert!(body.contains("\"ok\":true"));
        assert!(body.contains("\"code\":\"RS\""));
    }

    #[test]
    fn registry_step_endpoint_accepts_oci_id() {
        let (_, body) =
            handle_api_request("GET", "/v1/registry/step/OCI-1-48RS-327", "", &config());
        assert!(body.contains("\"ok\":true"));
        assert!(body.contains("\"stepNumber\":327"));
    }

    #[test]
    fn invalid_input_uses_error_envelope() {
        let (status, body) =
            handle_api_request("POST", "/v1/encode", "{\"space\":\"hex\"}", &config());
        assert_eq!(status, 400);
        assert!(body.contains("\"ok\":false"));
        assert!(body.contains("\"code\":\"invalid_input\""));
    }

    #[test]
    fn invalid_oci_id_uses_error_envelope() {
        let (status, body) = handle_api_request(
            "POST",
            "/v1/inspect",
            "{\"id\":\"OCI-1-48RS-999\"}",
            &config(),
        );
        assert_eq!(status, 400);
        assert!(body.contains("\"ok\":false"));
        assert!(body.contains("\"code\":\"invalid_step\""));
    }

    #[test]
    fn smoke_server_handles_health_over_http() -> Result<(), Box<dyn std::error::Error>> {
        let listener = match TcpListener::bind("127.0.0.1:0") {
            Ok(listener) => listener,
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                return Ok(());
            }
            Err(error) => return Err(Box::new(error)),
        };
        let address = listener.local_addr().unwrap();
        let server = Server::from_listener(listener, None).unwrap();
        let thread_config = config();
        let handle = std::thread::spawn(move || serve_one_request(&server, &thread_config));

        let mut stream = TcpStream::connect(address).unwrap();
        stream
            .write_all(b"GET /v1/health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
            .unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();

        handle.join().unwrap().unwrap();
        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(response.contains("\"ok\":true"));
        assert!(response.contains("\"service\":\"oci-local-api\""));
        Ok(())
    }
}
