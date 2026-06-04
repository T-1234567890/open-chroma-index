# OCI CLI

A command-line client for Open Chroma Index.

## What is this?

This crate provides the `oci` command-line tool. It uses the Open Chroma Index
kernel from the root project.

For the standard, registry, color model, and Rust library, see the main project
README:

```text
../README.md
```

## Install

Install the latest `cli-v*` release on macOS or Linux:

```text
curl -fsSL https://raw.githubusercontent.com/T-1234567890/open-chroma-index/main/install.sh | bash
```

Install a specific CLI release:

```text
curl -fsSL https://raw.githubusercontent.com/T-1234567890/open-chroma-index/main/install.sh | bash -s -- --version cli-v0.1.0
```

Install to a custom directory:

```text
curl -fsSL https://raw.githubusercontent.com/T-1234567890/open-chroma-index/main/install.sh | bash -s -- --dir ~/.local/bin --force
```

Install system-wide:

```text
curl -fsSL https://raw.githubusercontent.com/T-1234567890/open-chroma-index/main/install.sh | bash -s -- --system
```

From source:

```text
cargo install --path cli
```

During development:

```text
cargo run -p oci-cli -- encode "#E85A9A" --space hex
```

Manual fallback:

1. Open the GitHub Releases page.
2. Choose the latest `cli-v*` release.
3. Download the asset for your platform.
4. Verify the matching `.sha256` file if possible.
5. Put the `oci` binary somewhere in your `PATH`.

Windows:

Download `oci-x86_64-pc-windows-msvc.zip` manually from GitHub Releases. A
PowerShell installer is not included yet.

## Basic Usage

```text
oci encode "#E85A9A" --space hex
oci inspect OCI-1-48RS-327
oci export OCI-1-48RS-327 --to hex,oklch,css
oci registry info
oci serve
oci config
```

## Command Structure

```text
oci encode <INPUT> --space <SPACE> [--format json|pretty] [--precision <N>] [--verify]
oci inspect <OCI_ID> [--format json|pretty] [--exports all|none|summary|<LIST>] [--verify]
oci export <OCI_ID> --to <TARGETS> [--format json|plain|pretty] [--verify]
oci convert <INPUT> --from <SPACE> --to <TARGETS> [--format json|plain|pretty] [--verify]
oci serve [--host <HOST>] [--port <PORT>] [--config <PATH>] [--json]
oci registry <SUBCOMMAND>
oci test <SUBCOMMAND>
oci validate <TARGET> [--type id|registry|color]
oci config [--path <TOML_PATH>]
```

## Local Kernel API

Start the Local Kernel API:

```text
oci serve
```

Default address:

```text
http://127.0.0.1:8765
```

Endpoint summary:

```text
GET  /v1/health
POST /v1/encode
POST /v1/inspect
POST /v1/export
POST /v1/convert
GET  /v1/registry/info
GET  /v1/registry/families
GET  /v1/registry/family/{indexOrCode}
GET  /v1/registry/step/{idOrStep}
```

The server is local by default and returns JSON envelopes for every endpoint.
For request and response examples, see the main project documentation:
[`../docs/local-api.md`](../docs/local-api.md).

## Config

The CLI supports TOML config.

Default config location:

```text
<oci-install-dir>/config.toml
```

Open interactive config:

```text
oci config
```

Use custom config:

```text
oci config --path ./path/to/config.toml
```

## Output Formats

`pretty` is default for humans.

`json` is available for automation.

`plain` is available for minimal scripting.

Pretty output shows clean export values plus a compact verification block.
Use `--verify` for detailed per-target status and round-trip errors.

## Relationship to the Kernel

The CLI is only a client. The actual color kernel is in the root crate.

## License

Apache-2.0.
