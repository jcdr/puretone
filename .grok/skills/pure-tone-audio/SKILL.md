---
name: pure-tone-audio
description: >
  Continuous mono sine generation for pure-tone via AAudio, atomic live
  parameters, phase accumulation, dB gain, and automatic output-device hotplug
  (Bluetooth). Use when changing tone math, the audio worker, AAudio FFI,
  speaker routing, sample rate, or when the user runs /pure-tone-audio.
metadata:
  short-description: "AAudio sine engine + device hotplug"
---

# pure-tone audio engine

## Architecture

```
UI thread                          Audio worker thread
─────────                          ───────────────────
sliders → set_frequency_hertz  →  AtomicU32 bits
       → set_amplitude_decibels → AtomicU32 bits
                                  AAudio data callback:
                                    read atomics → gain → sin(phase) → float buffer
                                    advance phase
                                  poll device fingerprint / error callback
                                  → request_reopen → close/reopen stream
```

- Start continuous tone in `SineAudioEngine::start` (app open).
- Mono, 48 kHz (`AUDIO_SAMPLE_RATE_HERTZ`).
- Shared state: `SharedToneParameters` with atomics (`Relaxed` is enough for UI→audio).
- Do **not** do audio work on the UI thread.

## Pure math (`audio_math.rs`) — ship and test these

```text
f = f_min * (f_max/f_min)^t     t ∈ [0,1]   log frequency
g = 10^(dB/20)                  dB ∈ [-80,0]
sample = sin(phase) * g
phase += 2π * f / sample_rate   wrap with % 2π when ≥ 2π
```

Unit tests must call **these real functions** (endpoints, geometric mean at t=0.5, round-trip, 0 dB → 1, −20 dB → 0.1, −80 dB → 1e-4, phase advance). Never re-implement the formulas only inside the test oracle.

## AAudio path (Android)

Direct FFI to `libaaudio` (not cpal/oboe in this project):

- Output, shared mode, PCM float, 1 channel, 48 kHz
- Low-latency performance mode
- Data callback fills mono `f32` frames
- Error callback: on disconnect / non-OK → `request_reopen`
- Hotplug loop: every ~400 ms fingerprint output devices via JNI; on change close+reopen without app restart
- Preserve phase across reopen when possible
- Short cooldown (~150 ms) between reopen attempts
- Host fallback: timed software oscillator (no real output) so non-Android builds still link

### Why reopen (not only rely on the OS)

Existing AAudio streams often **stay on the original device** when Bluetooth connects. Reopening without a fixed device id attaches to the current default route.

### Device fingerprint (`android_context.rs`)

- Requires handles stored in `android_main` via `store_android_native_handles`
- `AudioManager.getDevices(GET_DEVICES_OUTPUTS=2)`
- Hash of count + each device id and type
- Returns `None` on host or JNI failure (skip hotplug compare)

## Lifecycle

| Event | Behavior |
|---|---|
| App create | Spawn worker; open stream; start |
| Slider move | Atomically update f / dB; next callback uses new values |
| BT connect/disconnect | Fingerprint change or error → reopen |
| Drop engine | `request_stop`; join worker |

## Logging tags

Filter `adb logcat -s PureTone:V`. Useful messages:

- `AAudio stream started (device fingerprint …)`
- `Audio output devices changed (… -> …); reopening stream`
- `AAudio error callback: …`

## Failure modes

| Symptom | Likely cause | Fix |
|---|---|---|
| Silent after BT connect | Stream stuck on old device | Ensure fingerprint poll + reopen loop |
| Crackles on reopen | No cooldown / hard cut | Keep REOPEN_COOLDOWN; optional short fade later |
| Open fails first time | Device busy / format | Log `AAudio_convertResultToText`; retry shared mode |
| Wrong pitch | sample rate mismatch | Keep builder + math at 48000 |
| Amplitude “jumps” | Linear dB vs linear gain confusion | UI is dB; samples use `linear_gain_from_decibels` |
| Host test fails to link aaudio | Android-only module | Keep AAudio under `cfg(target_os = "android")` |
| Fingerprint always 0 / None | Handles not stored | Call `store_android_native_handles` before audio starts |

## Do not

- Block the AAudio callback (no alloc/locks/JNI in the data callback)
- Call JNI from the data callback for device enumeration (poll on worker thread only)
- Switch primary path to stereo unless product requirements change
