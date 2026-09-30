package com.jcdr.puretone;

import android.app.Application;

/** Process application. Present so release builds have app code for R8 to map. */
public final class PureToneApplication extends Application {
    @Override
    public void onCreate() {
        super.onCreate();
    }
}
