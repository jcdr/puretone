mod android_context;
mod app_ui;
mod audio_engine;
mod audio_math;
mod keep_screen_on;

pub use app_ui::PureToneApp;
pub use audio_math::{
    frequency_hertz_from_log_normalized, linear_gain_from_decibels, render_sine_sample,
    sine_sample_from_phase, AUDIO_SAMPLE_RATE_HERTZ, MAXIMUM_AMPLITUDE_DECIBELS,
    MAXIMUM_FREQUENCY_HERTZ, MINIMUM_AMPLITUDE_DECIBELS, MINIMUM_FREQUENCY_HERTZ,
};

fn run_pure_tone_app(native_options: eframe::NativeOptions) -> eframe::Result {
    eframe::run_native(
        "Pure Tone",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(PureToneApp::new()))),
    )
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(android_app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("PureTone"),
    );
    log::info!("PureTone android_main starting");

    android_context::store_android_native_handles(
        android_app.vm_as_ptr(),
        android_app.activity_as_ptr(),
    );
    android_context::store_android_app(android_app.clone());
    keep_screen_on::enable_keep_screen_on_from_activity(
        android_app.vm_as_ptr(),
        android_app.activity_as_ptr(),
    );

    let mut native_options = eframe::NativeOptions::default();
    native_options.viewport = egui::ViewportBuilder::default()
        .with_fullscreen(true)
        .with_decorations(false)
        .with_maximized(true);
    native_options.android_app = Some(android_app);
    native_options.renderer = eframe::Renderer::Glow;

    if let Err(error) = run_pure_tone_app(native_options) {
        log::error!("eframe run failed: {error:?}");
    }
}

#[cfg(not(target_os = "android"))]
pub fn run_desktop() -> eframe::Result {
    let mut native_options = eframe::NativeOptions::default();
    native_options.viewport = egui::ViewportBuilder::default()
        .with_inner_size([480.0, 800.0])
        .with_title("Pure Tone");
    run_pure_tone_app(native_options)
}
