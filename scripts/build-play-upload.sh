#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SECRETS_DIR="${REPO_ROOT}/.secrets"
SIGNING_ENV="${SECRETS_DIR}/signing.env"
OUT_DIR="${REPO_ROOT}/out"

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

if [[ ! -f "${SIGNING_ENV}" ]]; then
  echo "Missing ${SIGNING_ENV}"
  echo "Run: ./scripts/init-local-secrets.sh"
  exit 1
fi

# shellcheck disable=SC1090
source "${SIGNING_ENV}"

KEYSTORE_PATH="${UPLOAD_KEYSTORE_FILE:-${SECRETS_DIR}/upload.keystore}"
if [[ ! -f "${KEYSTORE_PATH}" ]]; then
  echo "Missing keystore: ${KEYSTORE_PATH}"
  echo "Run: ./scripts/init-local-secrets.sh"
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found"
  exit 1
fi
if ! command -v rustup >/dev/null 2>&1; then
  echo "rustup not found"
  exit 1
fi

rustup target add aarch64-linux-android >/dev/null

PREBUILT="$(ls -d "${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/"* 2>/dev/null | head -1)"
CLANG="${PREBUILT}/bin/aarch64-linux-android26-clang"
if [[ ! -x "${CLANG}" ]]; then
  echo "NDK clang not found under ${ANDROID_NDK_HOME}"
  exit 1
fi

echo "=== 1/3 Build native library (release, aarch64) ==="
(
  cd "${REPO_ROOT}"
  export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="${CLANG}"
  export CC_aarch64_linux_android="${CLANG}"
  export AR_aarch64_linux_android="${PREBUILT}/bin/llvm-ar"
  cargo build --release --target aarch64-linux-android --lib
)

SO_SRC="${REPO_ROOT}/target/aarch64-linux-android/release/libpure_tone.so"
if [[ ! -f "${SO_SRC}" ]]; then
  echo "Native library not found: ${SO_SRC}"
  exit 1
fi

JNI_DIR="${REPO_ROOT}/android/app/src/main/jniLibs/arm64-v8a"
mkdir -p "${JNI_DIR}"
cp "${SO_SRC}" "${JNI_DIR}/libpure_tone.so"
echo "Copied libpure_tone.so into android/app/src/main/jniLibs/arm64-v8a/"

echo "=== 2/3 Build Play APK + AAB with Gradle ==="
export PURETONE_UPLOAD_STORE_FILE="${KEYSTORE_PATH}"
export PURETONE_UPLOAD_STORE_PASSWORD="${UPLOAD_STORE_PASSWORD}"
export PURETONE_UPLOAD_KEY_ALIAS="${UPLOAD_KEY_ALIAS}"
export PURETONE_UPLOAD_KEY_PASSWORD="${UPLOAD_STORE_PASSWORD}"

printf 'sdk.dir=%s\n' "${ANDROID_HOME}" > "${REPO_ROOT}/android/local.properties"

(
  cd "${REPO_ROOT}/android"
  chmod +x ./gradlew
  ./gradlew :app:assembleRelease :app:bundleRelease --no-daemon
)

mkdir -p "${OUT_DIR}"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
APK_SRC="${REPO_ROOT}/android/app/build/outputs/apk/release/app-release.apk"
AAB_SRC="${REPO_ROOT}/android/app/build/outputs/bundle/release/app-release.aab"

if [[ ! -f "${APK_SRC}" || ! -f "${AAB_SRC}" ]]; then
  echo "Gradle outputs missing under android/app/build/outputs/"
  exit 1
fi

APK_DEST="${OUT_DIR}/PureTone-play-${STAMP}.apk"
AAB_DEST="${OUT_DIR}/PureTone-play-${STAMP}.aab"
cp "${APK_SRC}" "${APK_DEST}"
cp "${AAB_SRC}" "${AAB_DEST}"
chmod 600 "${APK_DEST}" "${AAB_DEST}" || true

echo
echo "=== 3/3 Done ==="
ls -la "${APK_DEST}" "${AAB_DEST}"
echo
echo "Upload to Play (Internal app sharing / closed testing), prefer AAB:"
echo "  ${AAB_DEST}"
echo
ADB_BIN="${ANDROID_HOME}/platform-tools/adb"
if [[ -x "${ADB_BIN}" ]]; then
  echo "Optional device install:"
  echo "  \"${ADB_BIN}\" install -r \"${APK_DEST}\""
  echo "  \"${ADB_BIN}\" shell am start -n com.jcdr.puretone/android.app.NativeActivity"
fi
