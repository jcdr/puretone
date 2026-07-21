#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE_DIR="${SCRIPT_DIR}/templates"
TARGET_DIR="$(pwd)"

if [[ ! -d "${TEMPLATE_DIR}" ]]; then
  echo "Templates not found next to this installer: ${TEMPLATE_DIR}"
  exit 1
fi

if [[ "${TARGET_DIR}" == "${SCRIPT_DIR}" ]] || [[ "${TARGET_DIR}" == "${SCRIPT_DIR}"/* ]]; then
  echo "Refuse to install into the public docs tree."
  echo "Create an empty directory elsewhere, cd into it, then run this script."
  exit 1
fi

if [[ -f "${TARGET_DIR}/scripts/build-signed-release-apk.sh" ]]; then
  echo "This directory already looks like a vault (build script present)."
  echo "Remove it or use a fresh directory."
  exit 1
fi

echo "Installing Pure Tone release-vault templates into:"
echo "  ${TARGET_DIR}"
echo

cp -a "${TEMPLATE_DIR}/." "${TARGET_DIR}/"

if [[ ! -f "${TARGET_DIR}/config.env" ]]; then
  cp "${TARGET_DIR}/config.env.example" "${TARGET_DIR}/config.env"
fi
if [[ ! -f "${TARGET_DIR}/signing.env" ]]; then
  cp "${TARGET_DIR}/signing.env.example" "${TARGET_DIR}/signing.env"
fi

chmod 700 "${TARGET_DIR}/scripts"/*.sh 2>/dev/null || true
chmod 600 "${TARGET_DIR}/signing.env" "${TARGET_DIR}/config.env" 2>/dev/null || true

if [[ ! -d "${TARGET_DIR}/.git" ]]; then
  git init
  echo "Initialized git repository in vault."
fi

cat <<'EOF'

Vault skeleton installed.

Next steps (in THIS directory only):
  1. Edit config.env   (public source URL / tag)
  2. Edit signing.env  (passwords and alias)   chmod 600
  3. ./scripts/create-upload-keystore.sh
  4. ./scripts/fetch-public-source.sh
  5. ./scripts/build-signed-release-apk.sh
  6. git add -A && git commit -m "Initialize release vault"
  7. Back up upload.keystore offline; save passwords in a password manager

Do not add a public GitHub remote for this vault.
Do not paste vault paths or passwords into the public Pure Tone repo or chat.

EOF
