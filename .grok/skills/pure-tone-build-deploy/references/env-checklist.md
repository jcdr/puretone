# Environment checklist

```bash
rustc --version
rustup target list --installed | grep aarch64-linux-android
cargo apk --version
echo "JAVA_HOME=$JAVA_HOME"; java -version
echo "ANDROID_HOME=$ANDROID_HOME"
ls "$ANDROID_NDK_HOME" | head
adb version
adb devices
```

Minimum:

- Rust stable
- Target `aarch64-linux-android`
- JDK 17+
- Android SDK platform-tools + platforms android-35 (or matching)
- NDK 26+ (27.0.12077973 known good)
- cargo-apk 0.10+
