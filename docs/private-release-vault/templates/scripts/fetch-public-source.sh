#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "${SCRIPT_DIR}/common.sh"

VAULT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
load_vault_env "${VAULT_ROOT}"
require_cmd git

WORK_PATH="${VAULT_ROOT}/${WORK_DIR}"
mkdir -p "$(dirname "${WORK_PATH}")"

if [[ -d "${WORK_PATH}/.git" ]]; then
  echo "Updating existing checkout ${WORK_PATH}"
  git -C "${WORK_PATH}" fetch --all --tags
  git -C "${WORK_PATH}" checkout "${PUBLIC_GIT_REF}"
  git -C "${WORK_PATH}" pull --ff-only || true
else
  echo "Cloning ${PUBLIC_GIT_URL} (${PUBLIC_GIT_REF}) into ${WORK_PATH}"
  git clone --branch "${PUBLIC_GIT_REF}" "${PUBLIC_GIT_URL}" "${WORK_PATH}" \
    || git clone "${PUBLIC_GIT_URL}" "${WORK_PATH}"
  git -C "${WORK_PATH}" checkout "${PUBLIC_GIT_REF}"
fi

echo "Public source ready at ${WORK_PATH}"
git -C "${WORK_PATH}" rev-parse --short HEAD
