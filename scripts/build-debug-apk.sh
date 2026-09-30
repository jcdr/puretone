#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${REPO_ROOT}"

usage() {
  echo "Usage: $0"
  echo "Build the debug APK: debug native libraries (arm64-v8a, armeabi-v7a, x86_64) packaged by Gradle."
  echo "The version is PURETONE_VERSION or scripts/next-version.sh --local. See docs/VERSIONING.md."
}

case "${1:-}" in
  "") ;;
  -h|--help) usage; exit 0 ;;
  *) usage >&2; exit 2 ;;
esac

JAVA_HOME_DEFAULT="${HOME}/.local/jdk-17"
if [[ ! -x "${JAVA_HOME_DEFAULT}/bin/java" ]]; then
  JAVA_HOME_DEFAULT="${HOME}/tools/jdk-17"
fi
export JAVA_HOME="${JAVA_HOME:-$JAVA_HOME_DEFAULT}"
export PATH="${HOME}/.cargo/bin:${JAVA_HOME}/bin:${PATH}"
export ANDROID_HOME="${ANDROID_HOME:-${HOME}/Android/Sdk}"
export ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-${ANDROID_HOME}}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-${ANDROID_HOME}/ndk/27.0.12077973}"
export ANDROID_NDK_ROOT="${ANDROID_NDK_ROOT:-${ANDROID_NDK_HOME}}"

# shellcheck source=scripts/lib-native.sh
source "${REPO_ROOT}/scripts/lib-native.sh"

if [[ -n "${PURETONE_VERSION:-}" ]]; then
  DEBUG_VERSION="${PURETONE_VERSION}"
else
  DEBUG_VERSION="$("${REPO_ROOT}/scripts/next-version.sh" --local)"
fi
export PURETONE_VERSION="${DEBUG_VERSION}"

echo "=== Debug version ${DEBUG_VERSION} ==="

echo "=== 1/2 Build native libraries (debug: arm64-v8a, armeabi-v7a, x86_64) ==="
build_native_libs debug

echo "=== 2/2 Build debug APK with Gradle ==="
printf 'sdk.dir=%s\n' "${ANDROID_HOME}" > "${REPO_ROOT}/android/local.properties"
(
  cd "${REPO_ROOT}/android"
  ./gradlew :app:assembleDebug --no-daemon -PpuretoneVersion="${DEBUG_VERSION}"
)

APK="${REPO_ROOT}/android/app/build/outputs/apk/debug/app-debug.apk"
if [[ ! -f "${APK}" ]]; then
  echo "Debug APK missing: ${APK}"
  exit 1
fi
echo "Debug APK (version ${DEBUG_VERSION}): ${APK}"
echo "Install with: ./scripts/install-debug-apk.sh"
