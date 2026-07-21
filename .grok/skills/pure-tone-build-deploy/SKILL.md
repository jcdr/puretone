---
name: pure-tone-build-deploy
description: >
  Build and ADB-deploy the pure-tone Rust/egui debug APK with cargo-apk.
  Use when installing toolchains, packaging debug APKs, fixing NDK/JDK/SDK env,
  running cargo apk, adb install, launching NativeActivity, or when the user
  runs /pure-tone-build-deploy. Also when builds fail for aarch64-linux-android.
  Release signing and Play upload are outside this public repository.
metadata:
  short-description: "cargo-apk + ADB debug build for Pure Tone"
---

# Pure Tone build and deploy (public source tree)

## Scope of this repository

This public tree supports **development and debug installs** only.

- **In scope:** `cargo test`, `cargo apk build --lib` (debug), `adb install` of debug APK  
- **Out of scope:** release keystores, signing passwords, Play Console upload automation  

Store release packaging is performed by the publisher’s **private** process, not from scripts in this repo.

## Required environment (debug)

| Variable | Typical value |
|---|---|
| `ANDROID_HOME` | `$HOME/Android/Sdk` |
| `ANDROID_NDK_HOME` | NDK side-by-side under the SDK |
| `JAVA_HOME` | JDK 17+ |
| `PATH` | cargo, platform-tools, JDK bin |

Also: Rust stable, target `aarch64-linux-android`, `cargo-apk`.

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
cargo apk build --lib
# Output: target/debug/apk/PureTone.apk
```

## Install on a device

```bash
./scripts/install-debug-apk.sh
```

Or:

```bash
adb install -r target/debug/apk/PureTone.apk
adb shell am start -n com.jcdr.puretone/android.app.NativeActivity
```

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
