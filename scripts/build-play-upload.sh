#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SECRETS_DIR="${REPO_ROOT}/.secrets"
SIGNING_ENV="${SECRETS_DIR}/signing.env"
OUT_DIR="${REPO_ROOT}/out"
CHANGELOG_DIR="${REPO_ROOT}/fastlane/metadata/android/en-GB/changelogs"

usage() {
  echo "Usage: $0 [--notes-file FILE] [VERSION]"
  echo "Build, sign and tag a Play release. VERSION defaults to PURETONE_VERSION or scripts/next-version.sh."
  echo "  --notes-file FILE  release notes (max 500 characters) saved as fastlane changelogs/VERSION.txt"
  echo "See docs/VERSIONING.md."
}

RELEASE_VERSION="${PURETONE_VERSION:-}"
NOTES_FILE=""
while (( $# > 0 )); do
  case "$1" in
    --notes-file)
      [[ $# -ge 2 ]] || { usage >&2; exit 2; }
      NOTES_FILE="$2"
      shift 2
      ;;
    -h|--help) usage; exit 0 ;;
    -*) usage >&2; exit 2 ;;
    *) RELEASE_VERSION="$1"; shift ;;
  esac
done

cd "${REPO_ROOT}"

if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "Working tree has uncommitted changes to tracked files; commit or stash them first."
  exit 1
fi

if [[ -z "${RELEASE_VERSION}" ]]; then
  RELEASE_VERSION="$("${REPO_ROOT}/scripts/next-version.sh")"
fi
if [[ ! "${RELEASE_VERSION}" =~ ^[1-9][0-9]{7,9}$ ]] || (( RELEASE_VERSION > 2100000000 )); then
  echo "Invalid version '${RELEASE_VERSION}': expected yymmddnn (8 to 10 digits, at most 2100000000)."
  exit 1
fi
RELEASE_TAG="v${RELEASE_VERSION}"
if git rev-parse -q --verify "refs/tags/${RELEASE_TAG}" >/dev/null; then
  echo "Tag ${RELEASE_TAG} already exists."
  exit 1
fi

APK_DEST="${OUT_DIR}/PureTone-${RELEASE_VERSION}.apk"
AAB_DEST="${OUT_DIR}/PureTone-${RELEASE_VERSION}.aab"
if [[ -e "${APK_DEST}" || -e "${AAB_DEST}" ]]; then
  echo "Output already exists: ${AAB_DEST} or ${APK_DEST}"
  exit 1
fi

CHANGELOG_FILE=""
if [[ -n "${NOTES_FILE}" ]]; then
  if [[ ! -f "${NOTES_FILE}" ]]; then
    echo "Notes file not found: ${NOTES_FILE}"
    exit 1
  fi
  NOTES_LENGTH="$(wc -m < "${NOTES_FILE}")"
  if (( NOTES_LENGTH > 500 )); then
    echo "Release notes have ${NOTES_LENGTH} characters; Play allows at most 500."
    exit 1
  fi
  CHANGELOG_FILE="${CHANGELOG_DIR}/${RELEASE_VERSION}.txt"
fi

echo "=== Release version ${RELEASE_VERSION} (tag ${RELEASE_TAG}) ==="

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

restore_version_files() {
  if [[ "${RELEASE_COMMITTED:-0}" != 1 ]]; then
    git -C "${REPO_ROOT}" checkout -- Cargo.toml Cargo.lock
    if [[ -n "${CHANGELOG_FILE}" ]]; then
      rm -f "${CHANGELOG_FILE}"
    fi
    echo "Build failed; restored Cargo.toml and Cargo.lock."
  fi
}
trap restore_version_files EXIT

echo "=== Set Cargo version ${RELEASE_VERSION}.0.0 ==="
sed -i -E '0,/^version = "[^"]*"/s//version = "'"${RELEASE_VERSION}"'.0.0"/' "${REPO_ROOT}/Cargo.toml"
cargo update --workspace --quiet
if ! grep -q "^version = \"${RELEASE_VERSION}.0.0\"" "${REPO_ROOT}/Cargo.toml"; then
  echo "Failed to set version in Cargo.toml"
  exit 1
fi
if [[ -n "${CHANGELOG_FILE}" ]]; then
  mkdir -p "${CHANGELOG_DIR}"
  cp "${NOTES_FILE}" "${CHANGELOG_FILE}"
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

echo "=== 1/4 Build native libraries (release: arm64-v8a, armeabi-v7a, x86_64) ==="
JNI_ROOT="${REPO_ROOT}/android/app/src/main/jniLibs"
rm -rf "${JNI_ROOT}"
(
  cd "${REPO_ROOT}"
  build_native_abi aarch64-linux-android aarch64-linux-android26-clang arm64-v8a
  build_native_abi armv7-linux-androideabi armv7a-linux-androideabi26-clang armeabi-v7a
  build_native_abi x86_64-linux-android x86_64-linux-android26-clang x86_64
)

echo "=== 2/4 Build Play APK + AAB with Gradle ==="
export PURETONE_UPLOAD_STORE_FILE="${KEYSTORE_PATH}"
export PURETONE_UPLOAD_STORE_PASSWORD="${UPLOAD_STORE_PASSWORD}"
export PURETONE_UPLOAD_KEY_ALIAS="${UPLOAD_KEY_ALIAS}"
export PURETONE_UPLOAD_KEY_PASSWORD="${UPLOAD_STORE_PASSWORD}"

printf 'sdk.dir=%s\n' "${ANDROID_HOME}" > "${REPO_ROOT}/android/local.properties"

(
  cd "${REPO_ROOT}/android"
  chmod +x ./gradlew
  ./gradlew :app:assembleRelease :app:bundleRelease --no-daemon -PpuretoneVersion="${RELEASE_VERSION}"
)

mkdir -p "${OUT_DIR}"
APK_SRC="${REPO_ROOT}/android/app/build/outputs/apk/release/app-release.apk"
AAB_SRC="${REPO_ROOT}/android/app/build/outputs/bundle/release/app-release.aab"

if [[ ! -f "${APK_SRC}" || ! -f "${AAB_SRC}" ]]; then
  echo "Gradle outputs missing under android/app/build/outputs/"
  exit 1
fi

cp "${APK_SRC}" "${APK_DEST}"
cp "${AAB_SRC}" "${AAB_DEST}"
chmod 600 "${APK_DEST}" "${AAB_DEST}" || true

echo "=== 3/4 Commit and tag ${RELEASE_TAG} (local only) ==="
git add Cargo.toml Cargo.lock
if [[ -n "${CHANGELOG_FILE}" ]]; then
  git add "${CHANGELOG_FILE}"
fi
git commit -q -m "Release ${RELEASE_VERSION}"
RELEASE_COMMITTED=1
git tag -a "${RELEASE_TAG}" -m "Pure Tone ${RELEASE_VERSION}"
git log --oneline -1

echo
echo "=== 4/4 Done ==="
ls -la "${APK_DEST}" "${AAB_DEST}"
echo
echo "Upload to Play, prefer AAB (release name: ${RELEASE_VERSION}):"
echo "  ${AAB_DEST}"
echo
echo "Nothing was pushed. After the upload is accepted, push with:"
echo "  git push origin $(git rev-parse --abbrev-ref HEAD) ${RELEASE_TAG}"
echo
ADB_BIN="${ANDROID_HOME}/platform-tools/adb"
if [[ -x "${ADB_BIN}" ]]; then
  echo "Optional device install:"
  echo "  \"${ADB_BIN}\" install -r \"${APK_DEST}\""
  echo "  \"${ADB_BIN}\" shell am start -n com.jcdr.puretone/android.app.NativeActivity"
fi
