#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "${SCRIPT_DIR}/common.sh"

VAULT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
load_vault_env "${VAULT_ROOT}"

JAVA_HOME_DEFAULT="${HOME}/.local/jdk-17"
if [[ ! -x "${JAVA_HOME_DEFAULT}/bin/keytool" ]]; then
  JAVA_HOME_DEFAULT="${HOME}/tools/jdk-17"
fi
export JAVA_HOME="${JAVA_HOME:-$JAVA_HOME_DEFAULT}"
export PATH="${JAVA_HOME}/bin:${PATH}"

require_cmd keytool

KEYSTORE_PATH="${VAULT_ROOT}/${UPLOAD_KEYSTORE_FILE}"

if [[ -e "${KEYSTORE_PATH}" ]]; then
  echo "Keystore already exists: ${KEYSTORE_PATH}"
  echo "Refusing to overwrite."
  exit 1
fi

if [[ "${UPLOAD_STORE_PASSWORD}" == CHANGE_ME* ]] \
  || [[ "${UPLOAD_KEY_PASSWORD}" == CHANGE_ME* ]]; then
  echo "Edit signing.env and replace CHANGE_ME passwords first."
  exit 1
fi

DNAME="CN=${CERT_CN}, OU=${CERT_OU}, O=${CERT_O}, L=${CERT_L}, ST=${CERT_ST}, C=${CERT_C}"

keytool -genkeypair \
  -keystore "${KEYSTORE_PATH}" \
  -alias "${UPLOAD_KEY_ALIAS}" \
  -keyalg RSA \
  -keysize 2048 \
  -validity "${CERT_VALIDITY_DAYS}" \
  -storetype PKCS12 \
  -storepass "${UPLOAD_STORE_PASSWORD}" \
  -keypass "${UPLOAD_KEY_PASSWORD}" \
  -dname "${DNAME}"

chmod 600 "${KEYSTORE_PATH}"
echo "Created ${KEYSTORE_PATH}"
echo "Back it up offline. Store passwords in a password manager."
