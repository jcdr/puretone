# Module map (pure-tone)

| File | Role |
|---|---|
| `instructions.md` | Product requirements (source of truth for goals) |
| `Cargo.toml` | eframe feature split; cargo-apk android metadata |
| `src/lib.rs` | `android_main`, desktop stub, module tree |
| `src/app_ui.rs` | `PureToneApp`, custom vertical sliders, dark theme |
| `src/audio_math.rs` | Log frequency, dB gain, sine/phase; unit tests |
| `src/audio_engine.rs` | Worker, atomics, AAudio FFI, hotplug reopen |
| `src/android_context.rs` | Global JNI handles; output device fingerprint |
| `src/keep_screen_on.rs` | Window `FLAG_KEEP_SCREEN_ON` + `setKeepScreenOn` |

## Package IDs

- Application id: `com.jcdr.puretone`
- APK name: `PureTone`
- Native activity: `android.app.NativeActivity`
- Log tag: `PureTone`
