use std::sync::OnceLock;

struct AndroidNativeHandles {
    java_vm: *mut std::ffi::c_void,
    activity: *mut std::ffi::c_void,
}

unsafe impl Send for AndroidNativeHandles {}
unsafe impl Sync for AndroidNativeHandles {}

static ANDROID_NATIVE_HANDLES: OnceLock<AndroidNativeHandles> = OnceLock::new();

pub fn store_android_native_handles(java_vm: *mut std::ffi::c_void, activity: *mut std::ffi::c_void) {
    let _ = ANDROID_NATIVE_HANDLES.set(AndroidNativeHandles { java_vm, activity });
}

#[cfg(target_os = "android")]
pub fn output_audio_device_fingerprint() -> Option<u64> {
    use jni::objects::{JObject, JObjectArray, JValue};
    use jni::JavaVM;

    let handles = ANDROID_NATIVE_HANDLES.get()?;
    if handles.java_vm.is_null() || handles.activity.is_null() {
        return None;
    }

    let java_vm = unsafe { JavaVM::from_raw(handles.java_vm as *mut _) }.ok()?;
    let mut environment = java_vm.attach_current_thread().ok()?;
    let activity = unsafe { JObject::from_raw(handles.activity as *mut _) };

    (|| -> Result<u64, jni::errors::Error> {
        let service_name = environment.new_string("audio")?;
        let audio_service = environment
            .call_method(
                &activity,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[JValue::Object(&service_name)],
            )?
            .l()?;
        if audio_service.is_null() {
            return Ok(0);
        }

        const GET_DEVICES_OUTPUTS: i32 = 2;
        let devices_object = environment
            .call_method(
                &audio_service,
                "getDevices",
                "(I)[Landroid/media/AudioDeviceInfo;",
                &[JValue::Int(GET_DEVICES_OUTPUTS)],
            )?
            .l()?;
        if devices_object.is_null() {
            return Ok(0);
        }

        let devices = JObjectArray::from(devices_object);
        let device_count = environment.get_array_length(&devices)?;
        let mut fingerprint: u64 = device_count as u64;
        for index in 0..device_count {
            let device = environment.get_object_array_element(&devices, index)?;
            if device.is_null() {
                continue;
            }
            let device_id = environment
                .call_method(&device, "getId", "()I", &[])?
                .i()? as u64;
            let device_type = environment
                .call_method(&device, "getType", "()I", &[])?
                .i()? as u64;
            fingerprint = fingerprint
                .wrapping_mul(1_000_003)
                .wrapping_add(device_id.wrapping_mul(1_000_033).wrapping_add(device_type));
        }
        Ok(fingerprint)
    })()
    .ok()
}

#[cfg(not(target_os = "android"))]
pub fn output_audio_device_fingerprint() -> Option<u64> {
    None
}
