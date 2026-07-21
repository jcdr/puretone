# Pure Tone

Minimal full-screen Android app that generates a continuous mono sine wave.

**Publisher:** jcdr  
**Package:** `com.jcdr.puretone`  
**Support:** puretone.support@gmail.com  
**Source:** https://github.com/jcdr/puretone  
**Stack:** Rust, egui/eframe, AAudio

## Features

- Frequency **50–4000 Hz** on a logarithmic slider scale  
- Amplitude **−80–0 dB**  
- Dual vertical sliders, dark minimal UI  
- Live tone from launch; parameters update smoothly  
- Keeps the screen on while the app is focused  
- Automatic reopen of the audio stream when output devices change (e.g. Bluetooth)

## Development (this repository)

This repository is **source only**. It does not contain release keystores, signing passwords, or Play upload automation.

### Host checks

```bash
cargo test --lib
```

### Android debug APK (local device)

Requires Android NDK/SDK, Rust target `aarch64-linux-android`, and `cargo-apk`.

```bash
cargo apk build --lib
adb install -r target/debug/apk/PureTone.apk
adb shell am start -n com.jcdr.puretone/android.app.NativeActivity
```

Release signing and Play Store packaging are performed **outside** this repository by the publisher’s private release process.

To create that private release vault on your own machine (without recording its location in this repo), follow:

[docs/private-release-vault/README.md](docs/private-release-vault/README.md)

## Privacy

See [docs/privacy.md](docs/privacy.md).

## License

GNU General Public License v3.0 — see [LICENSE](LICENSE).
