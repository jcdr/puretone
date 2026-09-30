---
name: pure-tone
description: >
  Master skill for the pure-tone minimal Rust/egui Android sine-wave generator.
  Use when working on this repo, extending the sine app, reading instructions.md,
  planning changes, or when the user mentions pure-tone, Pure Tone app,
  com.jcdr.puretone, or runs /pure-tone. Routes to specialized sibling skills
  for build, audio, UI, style, and failure modes.
metadata:
  short-description: "pure-tone product map and skill router"
---

# pure-tone master skill

Load this skill first for any work in this repository. Then load the specialized skill that matches the change.

## Product (from `instructions.md`)

Minimal Android smartphone app that generates an audio sine wave:

| Requirement | Implementation fact |
|---|---|
| Frequency 20–20000 Hz, log-corrected | `audio_math::frequency_hertz_from_log_normalized` |
| Amplitude −100–0 dB | Linear-in-dB slider; `linear_gain_from_decibels` for sample scale |
| Fullscreen, keep screen on while focused | Theme + viewport fullscreen; JNI `FLAG_KEEP_SCREEN_ON` |
| Two vertical sliders, each ~half width | Custom painted sliders in `app_ui.rs` |
| Labels on top, comfortable font | Name + live value, 28 pt |
| Minimal dark UI | Dark panel, gray rails, white handles |
| Rust + egui | `eframe` 0.31 + glow, `android-native-activity` |
| No comments; explicit names | Enforced in code-style skill |
| Commit often, small diffs | One concern per commit |
| Deploy via USB ADB | `cargo apk` + `adb install -r` |

User-agreed defaults from planification:

- Continuous live tone from launch (no play/stop)
- Labels: name + live value
- Dark minimal UI
- Mono 48 kHz
- Phone ready over USB ADB

## Module map

```
src/lib.rs              android_main, eframe entry
src/app_ui.rs           PureToneApp, custom vertical sliders
src/audio_math.rs       log f, dB gain, sine samples + unit tests
src/audio_engine.rs     worker thread, AAudio + hotplug
src/android_context.rs  OnceLock JNI handles, device fingerprint
src/keep_screen_on.rs   Window flags via JNI
```

Package: `com.jcdr.puretone` · APK label: `Pure Tone` · Activity: `android.app.NativeActivity`

## Specialized skills (load by task)

| Task | Skill |
|---|---|
| Toolchain, cargo-apk, ADB install/launch | `pure-tone-build-deploy` |
| Sine engine, AAudio, Bluetooth routing | `pure-tone-audio` |
| Layout, custom sliders, keep-screen-on UI | `pure-tone-ui` |
| Naming, no comments, tests, commits | `pure-tone-code-style` |
| Debug crashes, silent audio, white rails, etc. | `pure-tone-failure-modes` |

## Non-goals (do not add unless asked)

- Play/stop toggle or drag-only tone
- Stereo or non-48 kHz as primary path
- Release signing or Play upload automation inside this public tree
- Desktop/web as primary deliverable
- Comments in source

## Change workflow

1. Read `instructions.md` and this skill.
2. Load the matching specialized skill.
3. Keep pure math in `audio_math.rs` with host `cargo test --lib`.
4. Verify on device when audio/UI/lifecycle changes: build → install → logcat → optional screencap.
5. Commit small, explicit messages.

## Quick commands

```bash
export PATH="$HOME/.cargo/bin:$HOME/.local/jdk-17/bin:$ANDROID_HOME/platform-tools:$PATH"
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/27.0.12077973}"
export JAVA_HOME="${JAVA_HOME:-$HOME/.local/jdk-17}"

cargo test --lib
cargo apk build --lib
adb install -r target/debug/apk/PureTone.apk
adb shell am start -n com.jcdr.puretone/android.app.NativeActivity
adb logcat -s PureTone:V
```
