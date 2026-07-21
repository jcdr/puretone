#[cfg(target_os = "android")]
pub fn enable_keep_screen_on_from_activity(
    java_vm: *mut std::ffi::c_void,
    activity: *mut std::ffi::c_void,
) {
    use jni::objects::{JObject, JValue};
    use jni::sys::jint;
    use jni::JavaVM;

    if java_vm.is_null() || activity.is_null() {
        log::warn!("null handles for keep-screen-on");
        return;
    }

    const FLAG_KEEP_SCREEN_ON: jint = 128;

    let java_vm = match unsafe { JavaVM::from_raw(java_vm as *mut _) } {
        Ok(vm) => vm,
        Err(error) => {
            log::error!("JavaVM::from_raw failed: {error:?}");
            return;
        }
    };

    let mut environment = match java_vm.attach_current_thread() {
        Ok(environment) => environment,
        Err(error) => {
            log::error!("attach_current_thread failed: {error:?}");
            return;
        }
    };

    let activity_object = unsafe { JObject::from_raw(activity as *mut _) };

    let result = (|| -> Result<(), jni::errors::Error> {
        let window = environment
            .call_method(
                &activity_object,
                "getWindow",
                "()Landroid/view/Window;",
                &[],
            )?
            .l()?;
        environment.call_method(
            &window,
            "addFlags",
            "(I)V",
            &[JValue::Int(FLAG_KEEP_SCREEN_ON)],
        )?;
        let decor_view = environment
            .call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?
            .l()?;
        environment.call_method(
            &decor_view,
            "setKeepScreenOn",
            "(Z)V",
            &[JValue::Bool(jni::sys::JNI_TRUE)],
        )?;
        Ok(())
    })();

    match result {
        Ok(()) => {
            log::info!("FLAG_KEEP_SCREEN_ON and setKeepScreenOn applied");
        }
        Err(error) => {
            log::error!("enable_keep_screen_on failed: {error:?}");
        }
    }
}

#[cfg(not(target_os = "android"))]
#[allow(dead_code)]
pub fn enable_keep_screen_on_from_activity(
    _java_vm: *mut std::ffi::c_void,
    _activity: *mut std::ffi::c_void,
) {
}
