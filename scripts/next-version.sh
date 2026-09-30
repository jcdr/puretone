#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PLAY_MAX_VERSION_CODE=2100000000
PRE_SCHEME_MAX_VERSION_CODE=2

usage() {
  echo "Usage: $0 [--local]"
  echo "Print the next free Pure Tone version (UTC yymmddnn, see docs/VERSIONING.md)."
  echo "  --local  use local tags only, do not query origin"
}

fail() {
  echo "next-version: $*" >&2
  exit 1
}

use_remote=1
for arg in "$@"; do
  case "${arg}" in
    --local) use_remote=0 ;;
    -h|--help) usage; exit 0 ;;
    *) usage >&2; exit 2 ;;
  esac
done

cd "${REPO_ROOT}"

read -r utc_year utc_month utc_day < <(date -u '+%Y %m %d')
utc_year=$((10#${utc_year}))
utc_month=$((10#${utc_month}))
utc_day=$((10#${utc_day}))
(( utc_year >= 2000 )) || fail "UTC year ${utc_year} is before 2000; check the system clock"

day_base=$(( (utc_year - 2000) * 1000000 + utc_month * 10000 + utc_day * 100 ))

tag_names="$(git tag -l 'v[0-9]*')"
if (( use_remote )); then
  if remote_refs="$(GIT_TERMINAL_PROMPT=0 \
      GIT_SSH_COMMAND="${GIT_SSH_COMMAND:-ssh -o BatchMode=yes -o ConnectTimeout=10}" \
      timeout 30 git ls-remote --tags --refs origin 2>/dev/null)"; then
    tag_names+=$'\n'"$(awk '{ sub("^refs/tags/", "", $2); print $2 }' <<<"${remote_refs}")"
  else
    echo "next-version: warning: origin not reachable, using local tags only" >&2
  fi
fi

highest_code=0
highest_nn_today=-1
while IFS= read -r tag_name; do
  [[ "${tag_name}" =~ ^v([1-9][0-9]{0,9})$ ]] || continue
  tag_code=$((10#${BASH_REMATCH[1]}))
  (( tag_code > highest_code )) && highest_code=${tag_code}
  if (( tag_code >= day_base && tag_code <= day_base + 99 )); then
    (( tag_code - day_base > highest_nn_today )) && highest_nn_today=$(( tag_code - day_base ))
  fi
done <<<"${tag_names}"

next_nn=$(( highest_nn_today + 1 ))
(( next_nn <= 99 )) || fail "all 100 versions of UTC day $(( day_base / 100 )) are used (v$(( day_base + 99 )) exists); wait for the next UTC day"

next_code=$(( day_base + next_nn ))

floor_code=${PRE_SCHEME_MAX_VERSION_CODE}
(( highest_code > floor_code )) && floor_code=${highest_code}
(( next_code > floor_code )) || fail "next version ${next_code} is not greater than highest used version ${floor_code}; check the system clock and tags"
(( next_code <= PLAY_MAX_VERSION_CODE )) || fail "next version ${next_code} exceeds the Play versionCode limit ${PLAY_MAX_VERSION_CODE}"

echo "${next_code}"
