# Pure Tone

Minimal full-screen Android app that generates a continuous mono sine wave.

**Publisher:** jcdr  
**Package:** `com.jcdr.puretone`  
**Support:** puretone.support@gmail.com  
**Source:** https://github.com/jcdr/puretone  
**Stack:** Rust, egui/eframe, AAudio

## Features

- Frequency **20–20000 Hz** on a logarithmic slider scale  
- Amplitude **−100–0 dB**  
- Dual vertical sliders, dark minimal UI  
- Live tone from launch; parameters update smoothly  
- Keeps the screen on while the app is focused  
- Automatic reopen of the audio stream when output devices change (e.g. Bluetooth)

## Behaviour

The tone plays only while the app is on screen. About 60 ms after you switch to another app, the lock screen comes on or the app's window is closed, it fades out. When you return it resumes with the same frequency and amplitude. Split screen keeps playing.

Android throttles or freezes apps in the background and Pure Tone has no foreground service, so background playback would be unreliable. Developer details are next to the window check in `src/audio_engine.rs`.

## Development

### Host checks

```bash
cargo test --lib
```

### Debug APK (local device)

Requires the Android SDK and NDK (27.0.12077973), JDK 17, and the Rust targets `aarch64-linux-android`, `armv7-linux-androideabi` and `x86_64-linux-android`.

```bash
./scripts/build-debug-apk.sh
./scripts/install-debug-apk.sh
```

The debug build compiles the native library for the three ABIs and packages it with Gradle (`android/app/build/outputs/apk/debug/app-debug.apk`). The first install over an older cargo-apk build needs `adb uninstall com.jcdr.puretone`, because the signing key changed.

### Play-ready signed APK + AAB (local only)

Signing material lives under **`.secrets/`** on your machine. It is **gitignored** and must never be pushed to GitHub.

One-time:

```bash
# requires: apg, JDK keytool, Android SDK/NDK
./scripts/init-local-secrets.sh
```

Build (outputs in `out/`, also gitignored):

```bash
./scripts/build-play-upload.sh
```

The script picks the next version (`yymmddnn`, UTC date + daily counter), builds and signs `out/PureTone-<version>.aab` (and `.apk`), then commits `Release <version>` and tags `v<version>` locally; it never pushes. Upload the `.aab` in Play Console. See [docs/VERSIONING.md](docs/VERSIONING.md).

See [`.secrets/README.md`](.secrets/README.md).

## Privacy

See [docs/privacy.md](docs/privacy.md).

## License

GNU General Public License v3.0 — see [LICENSE](LICENSE).
