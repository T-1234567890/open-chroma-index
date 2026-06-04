#!/usr/bin/env bash
set -euo pipefail

REPO="${OCI_REPO:-T-1234567890/open-chroma-index}"
VERSION=""
INSTALL_DIR="${HOME}/.local/bin"
INSTALL_DIR_SET=0
SYSTEM_INSTALL=0
NO_CHECKSUM=0
FORCE=0

usage() {
  cat <<'EOF'
Install the Open Chroma Index CLI.

Usage:
  install.sh [OPTIONS]

Options:
  --version <tag>   Install a specific CLI release tag, for example cli-v0.1.0
  --dir <path>      Install into a custom directory
  --system          Install into /usr/local/bin, using sudo only when needed
  --no-checksum     Skip SHA-256 checksum verification
  --force           Overwrite an existing oci binary
  -h, --help        Show this help text

By default, the latest cli-v* GitHub Release is installed into ~/.local/bin.
Windows users should download the oci-x86_64-pc-windows-msvc.zip asset manually
from GitHub Releases.
EOF
}

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --version)
      [ "$#" -ge 2 ] || die "--version requires a tag"
      VERSION="$2"
      shift 2
      ;;
    --dir)
      [ "$#" -ge 2 ] || die "--dir requires a path"
      INSTALL_DIR="$2"
      INSTALL_DIR_SET=1
      shift 2
      ;;
    --system)
      SYSTEM_INSTALL=1
      if [ "$INSTALL_DIR_SET" -eq 0 ]; then
        INSTALL_DIR="/usr/local/bin"
      fi
      shift
      ;;
    --no-checksum)
      NO_CHECKSUM=1
      shift
      ;;
    --force)
      FORCE=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "unknown option: $1"
      ;;
  esac
done

need_cmd curl
need_cmd tar

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin)
    case "$ARCH" in
      arm64|aarch64) TARGET="aarch64-apple-darwin" ;;
      x86_64|amd64) TARGET="x86_64-apple-darwin" ;;
      *) die "unsupported macOS architecture: $ARCH" ;;
    esac
    ;;
  Linux)
    case "$ARCH" in
      x86_64|amd64) TARGET="x86_64-unknown-linux-gnu" ;;
      *) die "unsupported Linux architecture: $ARCH" ;;
    esac
    ;;
  MINGW*|MSYS*|CYGWIN*)
    cat >&2 <<EOF
Windows install script support is not implemented yet.

Download the Windows zip manually from:
https://github.com/${REPO}/releases

Asset:
oci-x86_64-pc-windows-msvc.zip
EOF
    exit 1
    ;;
  *)
    die "unsupported operating system: $OS"
    ;;
esac

ASSET="oci-${TARGET}.tar.gz"
CHECKSUM="oci-${TARGET}.sha256"

latest_cli_release() {
  curl -fsSL "https://api.github.com/repos/${REPO}/releases?per_page=100" |
    sed -n 's/.*"tag_name":[[:space:]]*"\(cli-v[^"]*\)".*/\1/p' |
    head -n 1
}

if [ -z "$VERSION" ]; then
  VERSION="$(latest_cli_release)"
  [ -n "$VERSION" ] || die "could not find a cli-v* release for ${REPO}"
fi

case "$VERSION" in
  cli-v*) ;;
  *) die "--version must be a CLI release tag such as cli-v0.1.0" ;;
esac

BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

printf 'Installing OCI CLI %s for %s\n' "$VERSION" "$TARGET"

curl -fL --retry 3 --output "${TMP_DIR}/${ASSET}" "${BASE_URL}/${ASSET}"

sha256_file() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print tolower($1)}'
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print tolower($1)}'
  else
    die "no SHA-256 tool found; install shasum/sha256sum or pass --no-checksum"
  fi
}

if [ "$NO_CHECKSUM" -eq 0 ]; then
  curl -fL --retry 3 --output "${TMP_DIR}/${CHECKSUM}" "${BASE_URL}/${CHECKSUM}"
  EXPECTED="$(awk '{print tolower($1)}' "${TMP_DIR}/${CHECKSUM}")"
  ACTUAL="$(sha256_file "${TMP_DIR}/${ASSET}")"
  [ -n "$EXPECTED" ] || die "checksum file is empty or invalid"
  [ "$EXPECTED" = "$ACTUAL" ] || die "checksum mismatch for ${ASSET}"
  printf 'Verified SHA-256 checksum\n'
else
  printf 'Skipping SHA-256 checksum verification\n'
fi

tar -xzf "${TMP_DIR}/${ASSET}" -C "$TMP_DIR"
BINARY="${TMP_DIR}/oci"
[ -f "$BINARY" ] || die "archive did not contain an oci binary"
chmod +x "$BINARY"

DEST="${INSTALL_DIR}/oci"
if [ -e "$DEST" ] && [ "$FORCE" -eq 0 ]; then
  die "${DEST} already exists; pass --force to overwrite"
fi

install_with_privilege() {
  if [ -w "$INSTALL_DIR" ]; then
    install -m 755 "$BINARY" "$DEST"
  else
    command -v sudo >/dev/null 2>&1 || die "${INSTALL_DIR} is not writable and sudo is not available"
    sudo install -m 755 "$BINARY" "$DEST"
  fi
}

if [ "$SYSTEM_INSTALL" -eq 1 ]; then
  if [ ! -d "$INSTALL_DIR" ]; then
    if [ -w "$(dirname "$INSTALL_DIR")" ]; then
      mkdir -p "$INSTALL_DIR"
    else
      command -v sudo >/dev/null 2>&1 || die "$(dirname "$INSTALL_DIR") is not writable and sudo is not available"
      sudo mkdir -p "$INSTALL_DIR"
    fi
  fi
  install_with_privilege
else
  mkdir -p "$INSTALL_DIR"
  install -m 755 "$BINARY" "$DEST"
fi

printf 'Installed oci to %s\n' "$DEST"

case ":${PATH}:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    cat <<EOF

Note: ${INSTALL_DIR} is not in your PATH.
Add this to your shell profile:

  export PATH="${INSTALL_DIR}:\$PATH"
EOF
    ;;
esac

if command -v "$DEST" >/dev/null 2>&1; then
  "$DEST" --version || "$DEST" --help
else
  "$DEST" --version || "$DEST" --help
fi
