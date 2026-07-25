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

## Development

### Host checks

```bash
cargo test --lib
```

### Debug APK (local device)

Requires Android NDK/SDK, Rust target `aarch64-linux-android`, and `cargo-apk`.

```bash
cargo apk build --lib
./scripts/install-debug-apk.sh
```

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

Upload `out/PureTone-play-….aab` (or `.apk`) in Play Console (e.g. Internal app sharing).

See [`.secrets/README.md`](.secrets/README.md).

## Privacy

See [docs/privacy.md](docs/privacy.md).

## License

GNU General Public License v3.0 — see [LICENSE](LICENSE).
