#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "${SCRIPT_DIR}/common.sh"

VAULT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
load_vault_env "${VAULT_ROOT}"

JAVA_HOME_DEFAULT="${HOME}/.local/jdk-17"
if [[ ! -x "${JAVA_HOME_DEFAULT}/bin/java" ]]; then
  JAVA_HOME_DEFAULT="${HOME}/tools/jdk-17"
fi
export JAVA_HOME="${JAVA_HOME:-$JAVA_HOME_DEFAULT}"
export PATH="${HOME}/.cargo/bin:${JAVA_HOME}/bin:${PATH}"
export ANDROID_HOME="${ANDROID_HOME:-${HOME}/Android/Sdk}"
export ANDROID_SDK_ROOT="${ANDROID_HOME}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-${ANDROID_HOME}/ndk/27.0.12077973}"
export NDK_HOME="${ANDROID_NDK_HOME}"
export ANDROID_NDK_ROOT="${ANDROID_NDK_HOME}"
export PATH="${ANDROID_HOME}/platform-tools:${PATH}"

require_cmd cargo
require_cmd git

WORK_PATH="${VAULT_ROOT}/${WORK_DIR}"
KEYSTORE_PATH="${VAULT_ROOT}/${UPLOAD_KEYSTORE_FILE}"
OUT_PATH="${VAULT_ROOT}/${OUT_DIR}"

if [[ ! -d "${WORK_PATH}/.git" && ! -f "${WORK_PATH}/Cargo.toml" ]]; then
  echo "Public source missing. Run ./scripts/fetch-public-source.sh first."
  exit 1
fi

if [[ ! -f "${KEYSTORE_PATH}" ]]; then
  echo "Missing ${KEYSTORE_PATH}. Run ./scripts/create-upload-keystore.sh first."
  exit 1
fi

if [[ "${UPLOAD_STORE_PASSWORD}" == CHANGE_ME* ]] \
  || [[ "${UPLOAD_KEY_PASSWORD}" == CHANGE_ME* ]]; then
  echo "Edit signing.env passwords before building."
  exit 1
fi

mkdir -p "${OUT_PATH}"

# Inject signing only into the ephemeral work tree (never commit).
SIGNING_BLOCK="$(cat <<EOF

[package.metadata.android.signing.release]
path = "${KEYSTORE_PATH}"
keystore_password = "${UPLOAD_STORE_PASSWORD}"
key_alias = "${UPLOAD_KEY_ALIAS}"
key_password = "${UPLOAD_KEY_PASSWORD}"
EOF
)"

CARGO_TOML="${WORK_PATH}/Cargo.toml"
BACKUP_TOML="${WORK_PATH}/Cargo.toml.vault-backup"

if grep -q '\[package.metadata.android.signing.release\]' "${CARGO_TOML}"; then
  echo "Work tree Cargo.toml already has signing metadata; aborting to avoid clobber."
  exit 1
fi

cp "${CARGO_TOML}" "${BACKUP_TOML}"
printf '%s\n' "${SIGNING_BLOCK}" >> "${CARGO_TOML}"

cleanup() {
  if [[ -f "${BACKUP_TOML}" ]]; then
    mv "${BACKUP_TOML}" "${CARGO_TOML}"
  fi
}
trap cleanup EXIT

(
  cd "${WORK_PATH}"
  if ! cargo apk --help >/dev/null 2>&1; then
    echo "cargo-apk not available. Install with: cargo install cargo-apk"
    exit 1
  fi
  cargo apk build --lib --release
)

SRC_APK="${WORK_PATH}/target/release/apk/${ANDROID_APK_NAME}.apk"
if [[ ! -f "${SRC_APK}" ]]; then
  echo "Expected APK not found: ${SRC_APK}"
  exit 1
fi

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
DEST_APK="${OUT_PATH}/${ANDROID_APK_NAME}-release-${STAMP}.apk"
cp "${SRC_APK}" "${DEST_APK}"
chmod 600 "${DEST_APK}" || true

echo
echo "Signed release APK:"
ls -la "${DEST_APK}"
echo
echo "Upload this file in Google Play Console (or convert to AAB in a later vault pipeline)."
echo "Install test (optional):"
echo "  adb install -r \"${DEST_APK}\""
echo "  adb shell am start -n ${ANDROID_APPLICATION_ID}/android.app.NativeActivity"
