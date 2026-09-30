# Launcher activity. It lives in the framework; this keeps the contract visible to R8.
-keep class android.app.NativeActivity { *; }

# Rust reaches these framework types through JNI
# (src/keep_screen_on.rs, src/android_context.rs). They are not app dex classes.
-keep class android.app.Activity {
    public java.lang.Object getSystemService(java.lang.String);
    public android.view.Window getWindow();
}
-keep class android.view.Window {
    public void addFlags(int);
    public android.view.View getDecorView();
}
-keep class android.view.View {
    public void setKeepScreenOn(boolean);
}
-keep class android.media.AudioManager {
    public android.media.AudioDeviceInfo[] getDevices(int);
}
-keep class android.media.AudioDeviceInfo {
    public int getId();
    public int getType();
}

-keep class com.jcdr.puretone.PureToneApplication { *; }
