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

rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android >/dev/null

PREBUILT="$(ls -d "${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/"* 2>/dev/null | head -1)"
if [[ ! -d "${PREBUILT}/bin" ]]; then
  echo "NDK clang not found under ${ANDROID_NDK_HOME}"
  exit 1
fi

# rust target | NDK clang name (API 26) | jniLibs ABI directory
build_native_abi() {
  local rust_target="$1"
  local clang_name="$2"
  local abi="$3"
  local clang="${PREBUILT}/bin/${clang_name}"
  if [[ ! -x "${clang}" ]]; then
    echo "NDK clang not found: ${clang}"
    exit 1
  fi
  local upper="${rust_target^^}"
  upper="${upper//-/_}"
  local linker_var="CARGO_TARGET_${upper}_LINKER"
  local cc_var="CC_${rust_target//-/_}"
  local ar_var="AR_${rust_target//-/_}"
  export "${linker_var}=${clang}"
  export "${cc_var}=${clang}"
  export "${ar_var}=${PREBUILT}/bin/llvm-ar"
  cargo build --release --target "${rust_target}" --lib
  local so_src="${REPO_ROOT}/target/${rust_target}/release/libpure_tone.so"
  if [[ ! -f "${so_src}" ]]; then
    echo "Native library not found: ${so_src}"
    exit 1
  fi
  local jni_dir="${JNI_ROOT}/${abi}"
  mkdir -p "${jni_dir}"
  cp "${so_src}" "${jni_dir}/libpure_tone.so"
  echo "Copied libpure_tone.so into android/app/src/main/jniLibs/${abi}/"
}

echo "=== 1/3 Build native libraries (release: arm64-v8a, armeabi-v7a, x86_64) ==="
JNI_ROOT="${REPO_ROOT}/android/app/src/main/jniLibs"
rm -rf "${JNI_ROOT}"
(
  cd "${REPO_ROOT}"
  build_native_abi aarch64-linux-android aarch64-linux-android26-clang arm64-v8a
  build_native_abi armv7-linux-androideabi armv7a-linux-androideabi26-clang armeabi-v7a
  build_native_abi x86_64-linux-android x86_64-linux-android26-clang x86_64
)

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
