# Environment checklist

```bash
rustc --version
rustup target list --installed | grep -E 'aarch64-linux-android|armv7-linux-androideabi|x86_64-linux-android'
echo "JAVA_HOME=$JAVA_HOME"; java -version
echo "ANDROID_HOME=$ANDROID_HOME"
ls "$ANDROID_NDK_HOME" | head
adb version
adb devices
```

Minimum:

- Rust stable
- Targets `aarch64-linux-android`, `armv7-linux-androideabi`, `x86_64-linux-android`
- JDK 17+
- Android SDK platform-tools + platforms android-35 (or matching)
- NDK 26+ (27.0.12077973 known good)
- No cargo-apk (debug APKs are packaged by Gradle)
