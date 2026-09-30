---
name: pure-tone-build-deploy
description: >
  Build and ADB-deploy the pure-tone Rust/egui debug APK (cargo + NDK clang, Gradle packaging).
  Use when installing toolchains, packaging debug APKs, fixing NDK/JDK/SDK env,
  running build-debug-apk.sh, adb install, launching NativeActivity, or when the user
  runs /pure-tone-build-deploy. Also when builds fail for aarch64-linux-android.
  Release signing and Play upload are outside this public repository.
metadata:
  short-description: "Gradle + ADB debug build for Pure Tone"
---

# Pure Tone build and deploy (public source tree)

## Scope of this repository

This public tree supports **development and debug installs** only.

- **In scope:** `cargo test`, `scripts/build-debug-apk.sh` (debug), `adb install` of debug APK  
- **Out of scope:** release keystores, signing passwords, Play Console upload automation  

Store release packaging is performed by the publisher’s **private** process, not from scripts in this repo.

## Required environment (debug)

| Variable | Typical value |
|---|---|
| `ANDROID_HOME` | `$HOME/Android/Sdk` |
| `ANDROID_NDK_HOME` | NDK side-by-side under the SDK |
| `JAVA_HOME` | JDK 17+ |
| `PATH` | cargo, platform-tools, JDK bin |

Also: Rust stable, targets `aarch64-linux-android`, `armv7-linux-androideabi`, `x86_64-linux-android`. cargo-apk is not used (see `docs/VERSIONING.md`).

## Package identity

| Item | Value |
|---|---|
| Application id | `com.jcdr.puretone` |
| APK name | `PureTone` |
| Label | Pure Tone |
| Activity | `android.app.NativeActivity` |

## Build debug APK

```bash
cargo test --lib
./scripts/build-debug-apk.sh
# Output: android/app/build/outputs/apk/debug/app-debug.apk
```

`scripts/build-debug-apk.sh` builds debug `libpure_tone.so` for arm64-v8a, armeabi-v7a and x86_64 with the NDK clang (`scripts/lib-native.sh`, shared with the Play build), copies them to `android/app/src/main/jniLibs/` and runs `./gradlew :app:assembleDebug -PpuretoneVersion=<version>`. The version is `PURETONE_VERSION` or `scripts/next-version.sh --local`; `Cargo.toml` is not touched. cargo-apk was dropped: its ≤ 255 limit per version part and its `version_code` override panic make the version scheme impossible (see `docs/VERSIONING.md`).

## Install on a device

```bash
./scripts/install-debug-apk.sh
```

Or:

```bash
adb install -r android/app/build/outputs/apk/debug/app-debug.apk
adb shell am start -n com.jcdr.puretone/android.app.NativeActivity
```

The Gradle debug key differs from the key of older cargo-apk builds and from the Play key. If `adb install` fails with `INSTALL_FAILED_UPDATE_INCOMPATIBLE`, run `adb uninstall com.jcdr.puretone` once (deletes the app data). Never do this on a device where the Play build must be kept.

## Logs

```bash
adb logcat -s PureTone:V
```

## Entry point

`android_main` must set `NativeOptions.android_app`, init logger tag `PureTone`, store native handles for audio device fingerprinting, and apply keep-screen-on early.

## Failure modes (debug)

| Symptom | Fix |
|---|---|
| winit unsupported on host | Host eframe needs wayland/x11 features |
| NDK/Java missing | Install NDK and JDK 17 |
| adb empty | USB debugging + authorize |
| Wrong package | Use `com.jcdr.puretone` |

## Do not

- Add release keystore paths or passwords to this repository  
- Document private vault locations in public docs  
- Commit APKs or screenshots  
