#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${REPO_ROOT}"

export PATH="${HOME}/.cargo/bin:${PATH}"
export ANDROID_HOME="${ANDROID_HOME:-${HOME}/Android/Sdk}"
export ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-${ANDROID_HOME}}"
if [[ -z "${ANDROID_NDK_HOME:-}" && -d "${ANDROID_HOME}/ndk/27.0.12077973" ]]; then
  export ANDROID_NDK_HOME="${ANDROID_HOME}/ndk/27.0.12077973"
fi
if [[ -n "${ANDROID_NDK_HOME:-}" ]]; then
  export ANDROID_NDK_ROOT="${ANDROID_NDK_ROOT:-${ANDROID_NDK_HOME}}"
fi

CARGO_VERSION="$(sed -n -E '0,/^version = "([^"]*)"/s//\1/p' Cargo.toml)"
if [[ ! "${CARGO_VERSION}" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+) ]]; then
  echo "Cannot read version from Cargo.toml"
  exit 1
fi

if (( BASH_REMATCH[1] <= 255 && BASH_REMATCH[2] <= 255 && BASH_REMATCH[3] <= 255 )); then
  cargo apk build --lib "$@"
  exit 0
fi

BACKUP_DIR="$(mktemp -d)"
cp Cargo.toml Cargo.lock "${BACKUP_DIR}/"
restore_cargo_files() {
  cp "${BACKUP_DIR}/Cargo.toml" "${BACKUP_DIR}/Cargo.lock" "${REPO_ROOT}/"
  rm -rf "${BACKUP_DIR}"
}
trap restore_cargo_files EXIT

DEBUG_VERSION="0.0.0+${BASH_REMATCH[1]}"
echo "cargo-apk needs version parts <= 255; building with temporary version ${DEBUG_VERSION}"
sed -i -E '0,/^version = "[^"]*"/s//version = "'"${DEBUG_VERSION}"'"/' Cargo.toml
cargo apk build --lib "$@"
