---
name: pure-tone-failure-modes
description: >
  Failure-mode catalog and recovery playbook for pure-tone (build, deploy,
  UI, AAudio, Bluetooth, keep-screen-on, JNI). Use when debugging silent audio,
  crashes, wrong slider look, deploy failures, or when the user runs
  /pure-tone-failure-modes or says something is broken on device.
metadata:
  short-description: "pure-tone troubleshooting playbook"
---

# pure-tone failure modes

Investigate with **device logcat + host tests**. Prefer smallest repro: `cargo test --lib` then install debug APK and capture logs.

## Quick triage

```bash
cargo test --lib
adb devices
adb logcat -c
adb shell am force-stop com.jcdr.puretone
adb shell am start -n com.jcdr.puretone/android.app.NativeActivity
sleep 2
adb logcat -d -s PureTone:V
adb logcat -d | grep -iE 'AAudio|AndroidRuntime|FATAL|DEBUG|panic'
adb shell pidof com.jcdr.puretone
```

## Catalog

### Build / package

| Failure | Signals | Recovery |
|---|---|---|
| Host compile: winit unsupported | `compile_error!("The platform you're compiling for is not supported by winit")` | Enable eframe `wayland`+`x11` for non-Android target |
| No Java | sdkmanager / cargo-apk errors | User-local JDK 17; `JAVA_HOME` |
| Missing NDK | linker / clang not found | `sdkmanager "ndk;27.0.12077973"`; `ANDROID_NDK_HOME` |
| Need store release | not produced from this public tree | Publisher private release process |
| Wrong ABI | Installs but won't load native lib | `build_targets = ["aarch64-linux-android"]` for arm64 phones |

### Launch / lifecycle

| Failure | Signals | Recovery |
|---|---|---|
| Immediate exit | no pid; AndroidRuntime FATAL | logcat crash; verify `android_main` + `android_app` set |
| Missing android_app | eframe error about android_app | `native_options.android_app = Some(android_app)` |
| Black screen | process up, no UI | GLES/Adreno logs; try Glow renderer; check resume |
| Keep-screen-on fails | `addFlags … JavaException` | Call from `android_main` with activity ptr, not only later egui frame |

### Audio

| Failure | Signals | Recovery |
|---|---|---|
| No sound ever | no `AAudio stream started` | Check AAudio open/start errors; permissions not usually required for playback |
| Sound stops after BT connect | fingerprint log missing; stream not reopened | Ensure hotplug loop + `store_android_native_handles` before audio |
| Sound stays on phone speaker with BT connected | reopen not triggered | Confirm `getDevices` fingerprint changes in log; reopen on change |
| Disconnect glitch then silence | error callback without reopen | Error callback must `request_reopen`; worker must loop open |
| Wrong level | dB vs linear confusion | Gain only via `linear_gain_from_decibels` |
| Pitch wrong by ratio | sample rate | Builder and math both 48000 |
| Callback crash | native crash in audio thread | No JNI/alloc/lock in data callback |

### UI

| Failure | Signals | Recovery |
|---|---|---|
| Vertical slider tiny | knob only, short rail | Vertical length = `spacing.slider_width` for stock slider; use custom sized slider |
| Rail and knob both white | shared `inactive.bg_fill` | Custom paint: gray rail, white handle |
| Sliders not half-screen | single column / bad width | `ui.columns(2)` + thickness fraction of half width |
| Values don't update labels | format uses stale local | Read current f/dB each frame for labels |
| Hard to drag | small hit target | thickness ~50% of half width; large handle radius |

### Deploy / device

| Failure | Signals | Recovery |
|---|---|---|
| unauthorized | adb devices | Re-auth USB debugging |
| Success install but old UI | stale process | `am force-stop` then start; confirm new apk timestamp |
| Package not found on start | wrong component | `com.jcdr.puretone/android.app.NativeActivity` |

## Log line dictionary

| Line | Meaning |
|---|---|
| `PureTone android_main starting` | Native entry OK |
| `FLAG_KEEP_SCREEN_ON and setKeepScreenOn applied` | Screen-on OK |
| `AAudio stream started (device fingerprint X)` | Output stream running |
| `Audio output devices changed (A -> B); reopening stream` | Hotplug path working |
| `AAudio error callback: …` | Stream error; should reopen |
| `AAudio reopen failed: …` | Transient; worker retries after cooldown |

## Recovery order (audio routing)

1. Confirm process alive and stream started log.  
2. Toggle BT; watch for fingerprint change log within ~1 s.  
3. If no fingerprint change, debug JNI `getDevices` / stored handles.  
4. If fingerprint changes but no audio on new device, inspect reopen/open errors.  
5. Last resort: force-stop and relaunch (user-visible failure of hotplug).

## After fixing

- `cargo test --lib`
- Debug APK install + dual launch (start, force-stop, start)
- Commit only the fix with a focused message
