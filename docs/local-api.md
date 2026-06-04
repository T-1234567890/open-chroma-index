# Local Kernel API

The Open Chroma Index Local Kernel API is a local, language-agnostic HTTP JSON
API. It exposes the same OCI kernel used by the CLI so Python, JavaScript,
Swift, Go, Java, and other tools can call OCI without linking Rust directly and
without shelling out for every color conversion.

The Local Kernel API is started by the CLI:

```text
oci serve
```

Default address:

```text
http://127.0.0.1:8765
```

The server is local by default. It is not a cloud service, remote hosted API, or
telemetry service.

## Command

```text
oci serve [--host <HOST>] [--port <PORT>] [--config <PATH>] [--json]
```

Options:

- `--host <HOST>`: bind host. Default is `127.0.0.1`.
- `--port <PORT>`: bind port. Default is `8765`.
- `--config <PATH>`: optional TOML config path. This is equivalent to the
  global config-path behavior used by CLI commands.
- `--json`: print startup information as JSON.

Examples:

```text
oci serve
oci serve --port 9000
oci serve --config /tmp/oci.toml
oci serve --host 0.0.0.0 --port 8765
```

## Security Notes

OCI binds to `127.0.0.1` by default. This is intentional.

Do not bind to `0.0.0.0` unless another local device or process must connect
and the network exposure is understood. The CLI prints a warning when the server
is bound to anything other than `127.0.0.1`, `localhost`, or `::1`, unless
`server.warn_non_localhost` is disabled in config.

There is no authentication layer because the intended deployment is a local
developer process bound to localhost.

## Response Envelope

All endpoints return JSON.

Success:

```json
{
  "ok": true,
  "data": {}
}
```

Error:

```json
{
  "ok": false,
  "error": {
    "code": "invalid_input",
    "message": "Human-readable error message"
  }
}
```

Common error codes:

- `invalid_input`
- `invalid_id`
- `invalid_family`
- `invalid_step`
- `invalid_offset`
- `unsupported_space`
- `registry_error`
- `method_not_allowed`
- `not_found`
- `server_error`

## Endpoints

### `GET /v1/health`

Checks that the local server is running.

Response:

```json
{
  "ok": true,
  "data": {
    "status": "ok",
    "service": "oci-local-api",
    "version": "<server version>"
  }
}
```

### `POST /v1/encode`

Converts an input color to canonical OKLCH and encodes it as an OCI ID.

Request:

```json
{
  "input": "#E85A9A",
  "space": "hex",
  "format": "json",
  "roundToStep": false,
  "precision": 6
}
```

The API always returns JSON. The `format` field is accepted for client
compatibility, but CLI pretty/plain defaults do not affect API responses.

Response data uses the same structure as CLI JSON encode output. Abridged
example:

```json
{
  "ok": true,
  "data": {
    "input": "#E85A9A",
    "sourceSpace": "hex",
    "canonical": {
      "oklch": {
        "l": 0.669143,
        "c": 0.185968,
        "h": 355.308028
      },
      "oklab": {
        "l": 0.669143,
        "a": 0.185345,
        "b": -0.015212
      }
    },
    "oci": {
      "short": "OCI-1-48RS-327",
      "full": "OCI-1-48RS-A3-L09-C07",
      "precisionShort": "OCI-1-48RS-327@L-0.030857,C-0.010032,H+0.641361",
      "precisionFull": "OCI-1-48RS-A3-L09-C07@L-0.030857,C-0.010032,H+0.641361"
    },
    "offset": {
      "l": -0.030857,
      "c": -0.010032,
      "h": 0.641361,
      "string": "L-0.030857,C-0.010032,H+0.641361"
    },
    "exports": {
      "hex": {
        "status": "lossy",
        "value": "#E85A9A",
        "roundTripError": 0.000000348709
      },
      "oklch": {
        "l": 0.669143,
        "c": 0.185968,
        "h": 355.308028
      }
    },
    "support": [
      {
        "target": "hex",
        "status": "lossy",
        "roundTripError": 0.000000348709,
        "note": "HEX is always lossy because it quantizes to 8-bit sRGB"
      }
    ],
    "warnings": []
  }
}
```

The example ID is produced by the current bundled registry and should not be
treated as a future stable standard guarantee.

### `POST /v1/inspect`

Parses and decodes an OCI ID, applies any offset, and returns canonical color
data plus selected exports.

Request:

```json
{
  "id": "OCI-1-48RS-327",
  "exports": "summary"
}
```

`exports` may be `all`, `none`, `summary`, `list`, or a comma-separated target
list such as `hex,oklch,css`.

### `POST /v1/export`

Decodes an OCI ID and returns selected target representations.

Request:

```json
{
  "id": "OCI-1-48RS-327",
  "targets": ["hex", "oklch", "css", "display-p3"]
}
```

Supported targets match CLI export targets:

```text
hex
rgb
hsl
srgb
display-p3
adobe-rgb
rec709
oklch
oklab
css
json-token
swift
tailwind
cmyk
```

CMYK returns `profile_required` rather than numeric CMYK.

### `POST /v1/convert`

Converts an input color from a supported source space to selected target
exports. The implementation uses the same canonical OCI pipeline as the CLI.

Request:

```json
{
  "input": "232,90,154",
  "from": "rgb",
  "to": ["srgb", "display-p3", "oklch"]
}
```

Supported input spaces:

```text
hex
rgb
hsl
srgb
display-p3
adobe-rgb
rec709
oklch
oklab
oci
```

### `GET /v1/registry/info`

Returns registry version, family count, and step count.

Example response data:

```json
{
  "registry": {
    "version": "<registry version>",
    "familyCount": 64,
    "stepCount": 23040
  }
}
```

### `GET /v1/registry/families`

Returns all 64 registered families from the frozen bundled registry.

### `GET /v1/registry/family/{indexOrCode}`

Returns one family.

Examples:

```text
/v1/registry/family/48
/v1/registry/family/48RS
/v1/registry/family/RS
```

### `GET /v1/registry/step/{idOrStep}`

Returns one registered base step.

Examples:

```text
/v1/registry/step/OCI-1-48RS-327
/v1/registry/step/OCI-1-48RS-A3-L09-C07
```

## Curl Examples

Encode:

```bash
curl -s http://127.0.0.1:8765/v1/encode \
  -H 'content-type: application/json' \
  -d '{"input":"#E85A9A","space":"hex","precision":6}'
```

Inspect:

```bash
curl -s http://127.0.0.1:8765/v1/inspect \
  -H 'content-type: application/json' \
  -d '{"id":"OCI-1-48RS-327","exports":"summary"}'
```

Export:

```bash
curl -s http://127.0.0.1:8765/v1/export \
  -H 'content-type: application/json' \
  -d '{"id":"OCI-1-48RS-327","targets":["hex","oklch","css"]}'
```

Convert:

```bash
curl -s http://127.0.0.1:8765/v1/convert \
  -H 'content-type: application/json' \
  -d '{"input":"232,90,154","from":"rgb","to":["srgb","display-p3","oklch"]}'
```

Registry info:

```bash
curl -s http://127.0.0.1:8765/v1/registry/info
```

## Python Example

```python
import json
import urllib.request

payload = {"input": "#E85A9A", "space": "hex"}
request = urllib.request.Request(
    "http://127.0.0.1:8765/v1/encode",
    data=json.dumps(payload).encode(),
    headers={"content-type": "application/json"},
)

with urllib.request.urlopen(request) as response:
    result = json.load(response)

if not result["ok"]:
    raise RuntimeError(result["error"]["message"])

print(result["data"]["oci"]["precisionShort"])
```

## JavaScript Fetch Example

```js
const response = await fetch("http://127.0.0.1:8765/v1/encode", {
  method: "POST",
  headers: { "content-type": "application/json" },
  body: JSON.stringify({ input: "#E85A9A", space: "hex" }),
});

const result = await response.json();
if (!result.ok) {
  throw new Error(result.error.message);
}

console.log(result.data.oci.precisionShort);
```

## Implementation Notes

The Local Kernel API server lives in the CLI crate because it is a
developer-facing process, not part of the Rust kernel crate itself. The request
handlers call the same in-process command functions and JSON result builders
used by the CLI. They do not shell out to `oci`.

The root Rust crate remains the native Rust SDK. Use the Local Kernel API when
another language or process needs JSON access to OCI functionality.
