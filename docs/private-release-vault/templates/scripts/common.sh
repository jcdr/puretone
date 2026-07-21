#!/usr/bin/env bash
set -euo pipefail

vault_root() {
  local here
  here="$(cd "$(dirname "${BASH_SOURCE[1]}")/.." && pwd)"
  printf '%s\n' "${here}"
}

load_vault_env() {
  local root="$1"
  if [[ ! -f "${root}/config.env" ]]; then
    echo "Missing ${root}/config.env — copy from config.env.example"
    exit 1
  fi
  if [[ ! -f "${root}/signing.env" ]]; then
    echo "Missing ${root}/signing.env — copy from signing.env.example"
    exit 1
  fi
  # shellcheck disable=SC1091
  source "${root}/config.env"
  # shellcheck disable=SC1091
  source "${root}/signing.env"
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "Required command not found: $1"
    exit 1
  }
}
