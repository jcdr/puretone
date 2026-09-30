#!/usr/bin/env bash
set -euo pipefail

export PATH="${HOME}/Android/Sdk/platform-tools:${PATH}"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APK="${REPO_ROOT}/android/app/build/outputs/apk/debug/app-debug.apk"
PACKAGE_NAME="com.jcdr.puretone"

if [[ ! -f "${APK}" ]]; then
  echo "Missing ${APK}"
  echo "Build first with: ./scripts/build-debug-apk.sh"
  exit 1
fi

echo "Note: debug APKs are signed with the Gradle debug key. The first install over an older"
echo "cargo-apk debug build (or over the Play build) fails with INSTALL_FAILED_UPDATE_INCOMPATIBLE;"
echo "uninstall once with: adb uninstall ${PACKAGE_NAME} (this deletes the app data)."

adb devices -l
if ! adb install -r "${APK}"; then
  echo "Install failed. If the signature differs from the installed app, run:"
  echo "  adb uninstall ${PACKAGE_NAME}"
  echo "then run this script again."
  exit 1
fi
adb shell am start -n "${PACKAGE_NAME}/android.app.NativeActivity"
echo "Installed and started Pure Tone (debug)."
