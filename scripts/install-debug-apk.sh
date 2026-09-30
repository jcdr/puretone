#!/usr/bin/env bash
set -euo pipefail

export PATH="${HOME}/Android/Sdk/platform-tools:${PATH}"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APK="${REPO_ROOT}/target/debug/apk/PureTone.apk"

if [[ ! -f "${APK}" ]]; then
  echo "Missing ${APK}"
  echo "Build first with: ./scripts/build-debug-apk.sh"
  exit 1
fi

adb devices -l
adb install -r "${APK}"
adb shell am start -n com.jcdr.puretone/android.app.NativeActivity
echo "Installed and started Pure Tone (debug)."
