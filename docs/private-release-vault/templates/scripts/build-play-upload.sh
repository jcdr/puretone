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
export PATH="${ANDROID_HOME}/platform-tools:${ANDROID_HOME}/cmdline-tools/latest/bin:${PATH}"

require_cmd cargo
require_cmd git
require_cmd rustup

WORK_PATH="${VAULT_ROOT}/${WORK_DIR}"
KEYSTORE_PATH="${VAULT_ROOT}/${UPLOAD_KEYSTORE_FILE}"
OUT_PATH="${VAULT_ROOT}/${OUT_DIR}"

if [[ ! -f "${WORK_PATH}/Cargo.toml" ]]; then
  echo "Public source missing. Run ./scripts/fetch-public-source.sh first."
  exit 1
fi
if [[ ! -d "${WORK_PATH}/android" ]]; then
  echo "Public source has no android/ Gradle shell. Update puretone from GitHub and fetch again."
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

rustup target add aarch64-linux-android >/dev/null

echo "=== 1/3 Build native library (release, aarch64) ==="
PREBUILT="$(ls -d "${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/"* 2>/dev/null | head -1)"
CLANG="${PREBUILT}/bin/aarch64-linux-android26-clang"
if [[ ! -x "${CLANG}" ]]; then
  echo "NDK clang not found: ${CLANG}"
  exit 1
fi
(
  cd "${WORK_PATH}"
  export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="${CLANG}"
  export CC_aarch64_linux_android="${CLANG}"
  export AR_aarch64_linux_android="${PREBUILT}/bin/llvm-ar"
  cargo build --release --target aarch64-linux-android --lib
)

SO_SRC="${WORK_PATH}/target/aarch64-linux-android/release/libpure_tone.so"
if [[ ! -f "${SO_SRC}" ]]; then
  echo "Native library not found: ${SO_SRC}"
  exit 1
fi

JNI_DIR="${WORK_PATH}/android/app/src/main/jniLibs/arm64-v8a"
mkdir -p "${JNI_DIR}"
cp "${SO_SRC}" "${JNI_DIR}/libpure_tone.so"
echo "Copied libpure_tone.so into Gradle jniLibs"

echo "=== 2/3 Build Play-compatible APK + AAB with Gradle ==="
export PURETONE_UPLOAD_STORE_FILE="${KEYSTORE_PATH}"
export PURETONE_UPLOAD_STORE_PASSWORD="${UPLOAD_STORE_PASSWORD}"
export PURETONE_UPLOAD_KEY_ALIAS="${UPLOAD_KEY_ALIAS}"
# PKCS12 keystores only support one password; use store password for the key entry.
export PURETONE_UPLOAD_KEY_PASSWORD="${UPLOAD_STORE_PASSWORD}"

printf 'sdk.dir=%s\n' "${ANDROID_HOME}" > "${WORK_PATH}/android/local.properties"

(
  cd "${WORK_PATH}/android"
  if [[ ! -x ./gradlew ]]; then
    echo "gradlew missing in android/. Update public puretone repo and fetch again."
    exit 1
  fi
  chmod +x ./gradlew
  ./gradlew :app:assembleRelease :app:bundleRelease --no-daemon
)

mkdir -p "${OUT_PATH}"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"

APK_SRC="${WORK_PATH}/android/app/build/outputs/apk/release/app-release.apk"
AAB_SRC="${WORK_PATH}/android/app/build/outputs/bundle/release/app-release.aab"

if [[ ! -f "${APK_SRC}" ]]; then
  echo "Gradle APK missing: ${APK_SRC}"
  exit 1
fi
if [[ ! -f "${AAB_SRC}" ]]; then
  echo "Gradle AAB missing: ${AAB_SRC}"
  exit 1
fi

APK_DEST="${OUT_PATH}/PureTone-play-${STAMP}.apk"
AAB_DEST="${OUT_PATH}/PureTone-play-${STAMP}.aab"
cp "${APK_SRC}" "${APK_DEST}"
cp "${AAB_SRC}" "${AAB_DEST}"
chmod 600 "${APK_DEST}" "${AAB_DEST}" || true

echo
echo "=== 3/3 Done — upload ONE of these in Play Console (Internal app sharing) ==="
ls -la "${APK_DEST}" "${AAB_DEST}"
echo
echo "Prefer AAB if the form accepts both:"
echo "  ${AAB_DEST}"
echo "Or APK:"
echo "  ${APK_DEST}"
