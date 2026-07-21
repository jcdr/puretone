---
name: pure-tone-code-style
description: >
  Code style, testing, and commit discipline for pure-tone: no comments,
  explicit names, pure-math unit tests, small commits. Use when editing any
  source, adding tests, reviewing diffs, or when the user runs
  /pure-tone-code-style.
metadata:
  short-description: "No comments, explicit names, tests, commits"
---

# pure-tone code style

## Hard rules (from instructions)

1. **No comments** in source — no `//`, no `/* */`, no EOL comments.
2. **Explicit names** for constants, variables, functions, types (`frequency_hertz`, `amplitude_decibels`, `SineAudioEngine`, not `f`, `a`, `Eng`).
3. **Commit often** with **minimal diffs** and **appropriate messages** (one concern per commit).

## Structure preferences

- Pure, host-testable logic in `audio_math.rs`
- Platform/JNI/AAudio isolated under `cfg(target_os = "android")` where possible
- UI state separate from audio callback; share via atomics
- Prefer small focused functions over large mixed ones

## Testing

```bash
cargo test --lib
```

Requirements:

- Tests exercise **shipped** functions in `audio_math`
- Assert on real computed outputs (endpoints, geometric mid frequency, dB→gain, phase advance)
- Do **not** hardcode a fake oracle that reimplements the same formula only in the test
- Do **not** skip past the unit under test

When adding DSP/UI mapping logic that is pure, put it in `audio_math` (or another pure module) and test it on the host.

## Comment checker (for verification)

Treat only true comments as violations. Pointer deref lines like `*sample = …` are **not** comments.

```python
# Line is a comment if stripped startswith // or /*
# Also reject // outside of string literals
```

## Commit message style

Complete sentences, why + what, no trailer noise:

```text
Center thicker sliders in each half of the screen

Widen rails and handles from half-width fractions and place each
control in an equal column so it sits centered under its label.
```

Good commit slices for this project:

1. Scaffold + pure math + tests  
2. UI layout / slider paint  
3. Audio engine / hotplug  
4. Keep-screen-on / polish  
5. Style-only fixes  

## Naming examples (prefer)

| Avoid | Prefer |
|---|---|
| `f`, `g`, `p` | `frequency_hertz`, `linear_gain`, `phase_radians` |
| `run()` | `run_audio_worker`, `play_sine_stream_with_device_hotplug` |
| `State` | `SharedToneParameters`, `CallbackState` |
| `FLAG` | `FLAG_KEEP_SCREEN_ON` with value documented by name + const |

## cfg discipline

- Android-only symbols: gate with `cfg(target_os = "android")`
- Host stubs: `allow(dead_code)` only when the Android path is the sole caller
- Dual eframe features: host needs wayland/x11; Android needs android-native-activity

## Do not

- Leave explanatory comments “for the next person” — use names and structure
- Bundle unrelated refactors with feature commits
- Commit `target/`, APKs, keystores, or local screenshots
