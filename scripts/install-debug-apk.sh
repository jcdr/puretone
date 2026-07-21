#!/usr/bin/env bash
set -euo pipefail

export PATH="${HOME}/Android/Sdk/platform-tools:${PATH}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
APK="${REPO_ROOT}/target/debug/apk/PureTone.apk"

if [[ ! -f "${APK}" ]]; then
  echo "Missing ${APK}"
  echo "Build first with: cargo apk build --lib"
  exit 1
fi

adb devices -l
adb install -r "${APK}"
adb shell am start -n com.jcdr.puretone/android.app.NativeActivity
echo "Installed and started Pure Tone (debug)."
