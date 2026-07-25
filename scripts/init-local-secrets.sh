#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SECRETS_DIR="${REPO_ROOT}/.secrets"
SIGNING_ENV="${SECRETS_DIR}/signing.env"
KEYSTORE_PATH="${SECRETS_DIR}/upload.keystore"

JAVA_HOME_DEFAULT="${HOME}/.local/jdk-17"
if [[ ! -x "${JAVA_HOME_DEFAULT}/bin/keytool" ]]; then
  JAVA_HOME_DEFAULT="${HOME}/tools/jdk-17"
fi
export JAVA_HOME="${JAVA_HOME:-$JAVA_HOME_DEFAULT}"
export PATH="${JAVA_HOME}/bin:${PATH}"

if ! command -v apg >/dev/null 2>&1; then
  echo "apg not found. Install it (e.g. sudo apt install apg) and re-run."
  exit 1
fi
if ! command -v keytool >/dev/null 2>&1; then
  echo "keytool not found. Set JAVA_HOME to a JDK 17+ install."
  exit 1
fi

mkdir -p "${SECRETS_DIR}"

if [[ -e "${KEYSTORE_PATH}" ]] || [[ -e "${SIGNING_ENV}" ]]; then
  echo "Secrets already exist under ${SECRETS_DIR}"
  echo "Remove signing.env and upload.keystore manually if you intend to recreate them."
  exit 1
fi

PASSWORD="$(apg -a 1 -m 24 -x 24 -n 1 -M NCL)"
ALIAS="upload"
CERT_CN="Pure Tone"
CERT_OU="Release"
CERT_O="jcdr"
CERT_C="CH"
VALIDITY_DAYS="10000"
DNAME="CN=${CERT_CN}, OU=${CERT_OU}, O=${CERT_O}, C=${CERT_C}"

umask 077
cat > "${SIGNING_ENV}" <<EOF
UPLOAD_KEYSTORE_FILE="${KEYSTORE_PATH}"
UPLOAD_KEY_ALIAS="${ALIAS}"
UPLOAD_STORE_PASSWORD="${PASSWORD}"
UPLOAD_KEY_PASSWORD="${PASSWORD}"
EOF
chmod 600 "${SIGNING_ENV}"

keytool -genkeypair \
  -keystore "${KEYSTORE_PATH}" \
  -alias "${ALIAS}" \
  -keyalg RSA \
  -keysize 2048 \
  -validity "${VALIDITY_DAYS}" \
  -storetype PKCS12 \
  -storepass "${PASSWORD}" \
  -keypass "${PASSWORD}" \
  -dname "${DNAME}"

chmod 600 "${KEYSTORE_PATH}"

echo
echo "Created:"
echo "  ${SIGNING_ENV}"
echo "  ${KEYSTORE_PATH}"
echo
echo "Password was generated with apg and stored only in signing.env (not printed)."
echo "Back up both files offline. They are gitignored and must not reach GitHub."
echo
echo "Next: ./scripts/build-play-upload.sh"
