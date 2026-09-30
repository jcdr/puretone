# shellcheck shell=bash

NATIVE_ABIS=(
  "aarch64-linux-android aarch64-linux-android26-clang arm64-v8a"
  "armv7-linux-androideabi armv7a-linux-androideabi26-clang armeabi-v7a"
  "x86_64-linux-android x86_64-linux-android26-clang x86_64"
)

native_jni_root() {
  echo "${REPO_ROOT}/android/app/src/main/jniLibs"
}

native_prebuilt_dir() {
  ls -d "${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/"* 2>/dev/null | head -1 || true
}

build_native_abi() {
  local cargo_profile="$1"
  local rust_target="$2"
  local clang_name="$3"
  local abi="$4"
  local prebuilt
  prebuilt="$(native_prebuilt_dir)"
  local clang="${prebuilt}/bin/${clang_name}"
  if [[ ! -x "${clang}" ]]; then
    echo "NDK clang not found: ${clang}"
    exit 1
  fi
  local upper="${rust_target^^}"
  upper="${upper//-/_}"
  local cargo_profile_args=()
  if [[ "${cargo_profile}" == "release" ]]; then
    cargo_profile_args=(--release)
  fi
  (
    export "CARGO_TARGET_${upper}_LINKER=${clang}"
    export "CC_${rust_target//-/_}=${clang}"
    export "AR_${rust_target//-/_}=${prebuilt}/bin/llvm-ar"
    cd "${REPO_ROOT}"
    cargo build "${cargo_profile_args[@]}" --target "${rust_target}" --lib
  )
  local so_src="${REPO_ROOT}/target/${rust_target}/${cargo_profile}/libpure_tone.so"
  if [[ ! -f "${so_src}" ]]; then
    echo "Native library not found: ${so_src}"
    exit 1
  fi
  local jni_dir
  jni_dir="$(native_jni_root)/${abi}"
  mkdir -p "${jni_dir}"
  cp "${so_src}" "${jni_dir}/libpure_tone.so"
  echo "Copied ${cargo_profile} libpure_tone.so into android/app/src/main/jniLibs/${abi}/"
}

build_native_libs() {
  local cargo_profile="$1"
  if [[ "${cargo_profile}" != "debug" && "${cargo_profile}" != "release" ]]; then
    echo "build_native_libs: profile must be debug or release, got '${cargo_profile}'"
    exit 1
  fi
  if [[ -z "${ANDROID_NDK_HOME:-}" || ! -d "$(native_prebuilt_dir)/bin" ]]; then
    echo "NDK clang not found under ${ANDROID_NDK_HOME:-<ANDROID_NDK_HOME unset>}"
    exit 1
  fi
  local rust_targets=()
  local abi_entry
  for abi_entry in "${NATIVE_ABIS[@]}"; do
    rust_targets+=("${abi_entry%% *}")
  done
  rustup target add "${rust_targets[@]}" >/dev/null
  rm -rf "$(native_jni_root)"
  local rust_target clang_name abi
  for abi_entry in "${NATIVE_ABIS[@]}"; do
    read -r rust_target clang_name abi <<<"${abi_entry}"
    build_native_abi "${cargo_profile}" "${rust_target}" "${clang_name}" "${abi}"
  done
}
