use crate::audio_math::{
    advance_phase_radians, frequency_smoothing_coefficient, gain_smoothing_coefficient,
    linear_gain_from_decibels, sine_sample_from_phase, smooth_toward, AUDIO_SAMPLE_RATE_HERTZ,
};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

const FOREGROUND_POLL_INTERVAL: Duration = Duration::from_millis(100);

pub struct SharedToneParameters {
    frequency_hertz_bits: AtomicU32,
    amplitude_decibels_bits: AtomicU32,
    should_run: AtomicBool,
    reopen_requested: AtomicBool,
    output_muted: AtomicBool,
}

impl SharedToneParameters {
    pub fn new(frequency_hertz: f32, amplitude_decibels: f32) -> Arc<Self> {
        Arc::new(Self {
            frequency_hertz_bits: AtomicU32::new(frequency_hertz.to_bits()),
            amplitude_decibels_bits: AtomicU32::new(amplitude_decibels.to_bits()),
            should_run: AtomicBool::new(true),
            reopen_requested: AtomicBool::new(false),
            output_muted: AtomicBool::new(false),
        })
    }

    pub fn set_frequency_hertz(&self, frequency_hertz: f32) {
        self.frequency_hertz_bits
            .store(frequency_hertz.to_bits(), Ordering::Relaxed);
    }

    pub fn set_amplitude_decibels(&self, amplitude_decibels: f32) {
        self.amplitude_decibels_bits
            .store(amplitude_decibels.to_bits(), Ordering::Relaxed);
    }

    pub fn frequency_hertz(&self) -> f32 {
        f32::from_bits(self.frequency_hertz_bits.load(Ordering::Relaxed))
    }

    pub fn amplitude_decibels(&self) -> f32 {
        f32::from_bits(self.amplitude_decibels_bits.load(Ordering::Relaxed))
    }

    pub fn request_stop(&self) {
        self.should_run.store(false, Ordering::Relaxed);
        self.reopen_requested.store(true, Ordering::Relaxed);
    }

    pub fn should_run(&self) -> bool {
        self.should_run.load(Ordering::Relaxed)
    }

    pub fn request_reopen(&self) {
        self.reopen_requested.store(true, Ordering::Relaxed);
    }

    pub fn take_reopen_request(&self) -> bool {
        self.reopen_requested.swap(false, Ordering::Relaxed)
    }

    pub fn reopen_is_requested(&self) -> bool {
        self.reopen_requested.load(Ordering::Relaxed)
    }

    pub fn set_output_muted(&self, output_muted: bool) {
        self.output_muted.store(output_muted, Ordering::Relaxed);
    }

    pub fn output_is_muted(&self) -> bool {
        self.output_muted.load(Ordering::Relaxed)
    }
}

pub struct SineAudioEngine {
    shared_parameters: Arc<SharedToneParameters>,
    worker_handle: Option<JoinHandle<()>>,
}

impl SineAudioEngine {
    pub fn start(frequency_hertz: f32, amplitude_decibels: f32) -> Self {
        let shared_parameters = SharedToneParameters::new(frequency_hertz, amplitude_decibels);
        let worker_parameters = Arc::clone(&shared_parameters);
        let worker_handle = thread::Builder::new()
            .name("sine-audio".into())
            .spawn(move || run_audio_worker(worker_parameters))
            .expect("failed to spawn sine audio worker");
        Self {
            shared_parameters,
            worker_handle: Some(worker_handle),
        }
    }

    pub fn set_frequency_hertz(&self, frequency_hertz: f32) {
        self.shared_parameters.set_frequency_hertz(frequency_hertz);
    }

    pub fn set_amplitude_decibels(&self, amplitude_decibels: f32) {
        self.shared_parameters
            .set_amplitude_decibels(amplitude_decibels);
    }
}

impl Drop for SineAudioEngine {
    fn drop(&mut self) {
        self.shared_parameters.request_stop();
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

fn run_audio_worker(shared_parameters: Arc<SharedToneParameters>) {
    #[cfg(target_os = "android")]
    {
        if run_android_aaudio_worker(&shared_parameters) {
            return;
        }
        log::warn!("AAudio path failed; falling back to timed software oscillator");
    }
    run_software_timed_worker(shared_parameters);
}

fn run_software_timed_worker(shared_parameters: Arc<SharedToneParameters>) {
    let sample_rate = AUDIO_SAMPLE_RATE_HERTZ as f32;
    let gain_coefficient = gain_smoothing_coefficient(sample_rate);
    let frequency_coefficient = frequency_smoothing_coefficient(sample_rate);
    let frames_per_chunk: usize = 256;
    let chunk_duration =
        std::time::Duration::from_secs_f64(frames_per_chunk as f64 / sample_rate as f64);
    let mut phase_radians = 0.0_f32;
    let mut smoothed_linear_gain =
        linear_gain_from_decibels(shared_parameters.amplitude_decibels());
    let mut smoothed_frequency_hertz = shared_parameters.frequency_hertz();
    let mut sink_buffer = vec![0.0_f32; frames_per_chunk];

    while shared_parameters.should_run() {
        if !crate::android_context::activity_window_is_present() {
            thread::sleep(FOREGROUND_POLL_INTERVAL);
            continue;
        }
        let started = std::time::Instant::now();
        let target_frequency_hertz = shared_parameters.frequency_hertz();
        let target_linear_gain = linear_gain_from_decibels(shared_parameters.amplitude_decibels());
        for sample in sink_buffer.iter_mut() {
            smoothed_linear_gain =
                smooth_toward(smoothed_linear_gain, target_linear_gain, gain_coefficient);
            smoothed_frequency_hertz = smooth_toward(
                smoothed_frequency_hertz,
                target_frequency_hertz,
                frequency_coefficient,
            );
            *sample = sine_sample_from_phase(phase_radians) * smoothed_linear_gain;
            phase_radians =
                advance_phase_radians(phase_radians, smoothed_frequency_hertz, sample_rate);
        }
        std::hint::black_box(&sink_buffer);
        let elapsed = started.elapsed();
        if elapsed < chunk_duration {
            thread::sleep(chunk_duration - elapsed);
        }
    }
}

#[cfg(target_os = "android")]
fn run_android_aaudio_worker(shared_parameters: &Arc<SharedToneParameters>) -> bool {
    match android_aaudio::play_sine_stream_with_device_hotplug(shared_parameters) {
        Ok(()) => true,
        Err(error_message) => {
            log::error!("AAudio sine stream failed: {error_message}");
            false
        }
    }
}

#[cfg(target_os = "android")]
mod android_aaudio {
    use super::*;
    use crate::android_context;
    use std::ffi::c_void;
    use std::os::raw::{c_char, c_int};
    use std::time::Instant;

    const AAUDIO_OK: i32 = 0;
    const AAUDIO_DIRECTION_OUTPUT: i32 = 0;
    const AAUDIO_FORMAT_PCM_FLOAT: i32 = 2;
    const AAUDIO_SHARING_MODE_SHARED: i32 = 0;
    const AAUDIO_PERFORMANCE_MODE_LOW_LATENCY: i32 = 12;
    const AAUDIO_CALLBACK_RESULT_CONTINUE: i32 = 0;
    const AAUDIO_CALLBACK_RESULT_STOP: i32 = 1;
    const AAUDIO_ERROR_DISCONNECTED: i32 = -899;
    const DEVICE_POLL_INTERVAL: Duration = Duration::from_millis(400);
    const REOPEN_COOLDOWN: Duration = Duration::from_millis(150);
    // Gain smoothing time constant is 10 ms; 60 ms lets the tone fall to silence.
    const OUTPUT_MUTE_FADE_DURATION: Duration = Duration::from_millis(60);

    type AAudioStream = c_void;
    type AAudioStreamBuilder = c_void;

    type DataCallback = Option<
        unsafe extern "C" fn(
            stream: *mut AAudioStream,
            user_data: *mut c_void,
            audio_data: *mut c_void,
            num_frames: i32,
        ) -> i32,
    >;

    type ErrorCallback =
        Option<unsafe extern "C" fn(stream: *mut AAudioStream, user_data: *mut c_void, error: i32)>;

    #[link(name = "aaudio")]
    extern "C" {
        fn AAudio_createStreamBuilder(builder: *mut *mut AAudioStreamBuilder) -> i32;
        fn AAudioStreamBuilder_setDirection(builder: *mut AAudioStreamBuilder, direction: i32);
        fn AAudioStreamBuilder_setSharingMode(builder: *mut AAudioStreamBuilder, sharing_mode: i32);
        fn AAudioStreamBuilder_setFormat(builder: *mut AAudioStreamBuilder, format: i32);
        fn AAudioStreamBuilder_setChannelCount(
            builder: *mut AAudioStreamBuilder,
            channel_count: i32,
        );
        fn AAudioStreamBuilder_setSampleRate(builder: *mut AAudioStreamBuilder, sample_rate: i32);
        fn AAudioStreamBuilder_setPerformanceMode(
            builder: *mut AAudioStreamBuilder,
            performance_mode: i32,
        );
        fn AAudioStreamBuilder_setDataCallback(
            builder: *mut AAudioStreamBuilder,
            callback: DataCallback,
            user_data: *mut c_void,
        );
        fn AAudioStreamBuilder_setErrorCallback(
            builder: *mut AAudioStreamBuilder,
            callback: ErrorCallback,
            user_data: *mut c_void,
        );
        fn AAudioStreamBuilder_openStream(
            builder: *mut AAudioStreamBuilder,
            stream: *mut *mut AAudioStream,
        ) -> i32;
        fn AAudioStreamBuilder_delete(builder: *mut AAudioStreamBuilder) -> i32;
        fn AAudioStream_requestStart(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_requestStop(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_close(stream: *mut AAudioStream) -> i32;
        fn AAudio_convertResultToText(return_code: i32) -> *const c_char;
    }

    struct CallbackState {
        shared_parameters: Arc<SharedToneParameters>,
        phase_radians: f32,
        smoothed_linear_gain: f32,
        smoothed_frequency_hertz: f32,
        gain_smoothing_coefficient: f32,
        frequency_smoothing_coefficient: f32,
    }

    #[derive(Clone, Copy)]
    struct StreamCarryState {
        phase_radians: f32,
        smoothed_linear_gain: f32,
        smoothed_frequency_hertz: f32,
    }

    unsafe extern "C" fn sine_data_callback(
        _stream: *mut AAudioStream,
        user_data: *mut c_void,
        audio_data: *mut c_void,
        num_frames: i32,
    ) -> i32 {
        if user_data.is_null() || audio_data.is_null() || num_frames <= 0 {
            return AAUDIO_CALLBACK_RESULT_STOP;
        }
        let state = &mut *(user_data as *mut CallbackState);
        if !state.shared_parameters.should_run()
            || (state.shared_parameters.reopen_is_requested()
                && !state.shared_parameters.output_is_muted())
        {
            return AAUDIO_CALLBACK_RESULT_STOP;
        }
        let target_frequency_hertz = state.shared_parameters.frequency_hertz();
        let target_linear_gain = if state.shared_parameters.output_is_muted() {
            0.0
        } else {
            linear_gain_from_decibels(state.shared_parameters.amplitude_decibels())
        };
        let sample_rate = AUDIO_SAMPLE_RATE_HERTZ as f32;
        let output = std::slice::from_raw_parts_mut(audio_data as *mut f32, num_frames as usize);
        for sample in output.iter_mut() {
            state.smoothed_linear_gain = smooth_toward(
                state.smoothed_linear_gain,
                target_linear_gain,
                state.gain_smoothing_coefficient,
            );
            state.smoothed_frequency_hertz = smooth_toward(
                state.smoothed_frequency_hertz,
                target_frequency_hertz,
                state.frequency_smoothing_coefficient,
            );
            *sample = sine_sample_from_phase(state.phase_radians) * state.smoothed_linear_gain;
            state.phase_radians = advance_phase_radians(
                state.phase_radians,
                state.smoothed_frequency_hertz,
                sample_rate,
            );
        }
        AAUDIO_CALLBACK_RESULT_CONTINUE
    }

    unsafe extern "C" fn sine_error_callback(
        _stream: *mut AAudioStream,
        user_data: *mut c_void,
        error: i32,
    ) {
        if user_data.is_null() {
            return;
        }
        let state = &*(user_data as *const CallbackState);
        log::warn!("AAudio error callback: {} ({})", result_text(error), error);
        if error == AAUDIO_ERROR_DISCONNECTED || error != AAUDIO_OK {
            state.shared_parameters.request_reopen();
        }
    }

    fn result_text(code: i32) -> String {
        unsafe {
            let pointer = AAudio_convertResultToText(code);
            if pointer.is_null() {
                return format!("aaudio_error_{code}");
            }
            std::ffi::CStr::from_ptr(pointer)
                .to_string_lossy()
                .into_owned()
        }
    }

    fn open_and_start_stream(
        shared_parameters: &Arc<SharedToneParameters>,
        carry_state: StreamCarryState,
    ) -> Result<(*mut AAudioStream, *mut CallbackState), String> {
        unsafe {
            let mut builder: *mut AAudioStreamBuilder = std::ptr::null_mut();
            let create_code = AAudio_createStreamBuilder(&mut builder);
            if create_code != AAUDIO_OK || builder.is_null() {
                return Err(format!(
                    "AAudio_createStreamBuilder failed: {}",
                    result_text(create_code)
                ));
            }

            AAudioStreamBuilder_setDirection(builder, AAUDIO_DIRECTION_OUTPUT);
            AAudioStreamBuilder_setSharingMode(builder, AAUDIO_SHARING_MODE_SHARED);
            AAudioStreamBuilder_setFormat(builder, AAUDIO_FORMAT_PCM_FLOAT);
            AAudioStreamBuilder_setChannelCount(builder, 1);
            AAudioStreamBuilder_setSampleRate(builder, AUDIO_SAMPLE_RATE_HERTZ as c_int);
            AAudioStreamBuilder_setPerformanceMode(builder, AAUDIO_PERFORMANCE_MODE_LOW_LATENCY);

            let sample_rate = AUDIO_SAMPLE_RATE_HERTZ as f32;
            let callback_state = Box::new(CallbackState {
                shared_parameters: Arc::clone(shared_parameters),
                phase_radians: carry_state.phase_radians,
                smoothed_linear_gain: carry_state.smoothed_linear_gain,
                smoothed_frequency_hertz: carry_state.smoothed_frequency_hertz,
                gain_smoothing_coefficient: gain_smoothing_coefficient(sample_rate),
                frequency_smoothing_coefficient: frequency_smoothing_coefficient(sample_rate),
            });
            let callback_state_pointer = Box::into_raw(callback_state);
            AAudioStreamBuilder_setDataCallback(
                builder,
                Some(sine_data_callback),
                callback_state_pointer as *mut c_void,
            );
            AAudioStreamBuilder_setErrorCallback(
                builder,
                Some(sine_error_callback),
                callback_state_pointer as *mut c_void,
            );

            let mut stream: *mut AAudioStream = std::ptr::null_mut();
            let open_code = AAudioStreamBuilder_openStream(builder, &mut stream);
            let _ = AAudioStreamBuilder_delete(builder);
            if open_code != AAUDIO_OK || stream.is_null() {
                drop(Box::from_raw(callback_state_pointer));
                return Err(format!(
                    "AAudioStreamBuilder_openStream failed: {}",
                    result_text(open_code)
                ));
            }

            let start_code = AAudioStream_requestStart(stream);
            if start_code != AAUDIO_OK {
                let _ = AAudioStream_close(stream);
                drop(Box::from_raw(callback_state_pointer));
                return Err(format!(
                    "AAudioStream_requestStart failed: {}",
                    result_text(start_code)
                ));
            }

            Ok((stream, callback_state_pointer))
        }
    }

    fn close_stream(
        stream: *mut AAudioStream,
        callback_state_pointer: *mut CallbackState,
    ) -> StreamCarryState {
        unsafe {
            let carry_state = if !callback_state_pointer.is_null() {
                StreamCarryState {
                    phase_radians: (*callback_state_pointer).phase_radians,
                    smoothed_linear_gain: (*callback_state_pointer).smoothed_linear_gain,
                    smoothed_frequency_hertz: (*callback_state_pointer).smoothed_frequency_hertz,
                }
            } else {
                StreamCarryState {
                    phase_radians: 0.0,
                    smoothed_linear_gain: 0.0,
                    smoothed_frequency_hertz: 0.0,
                }
            };
            if !stream.is_null() {
                let _ = AAudioStream_requestStop(stream);
                let _ = AAudioStream_close(stream);
            }
            if !callback_state_pointer.is_null() {
                drop(Box::from_raw(callback_state_pointer));
            }
            carry_state
        }
    }

    fn sleep_while_running(shared_parameters: &SharedToneParameters, duration: Duration) {
        let started = Instant::now();
        while shared_parameters.should_run() && started.elapsed() < duration {
            let remaining = duration.saturating_sub(started.elapsed());
            thread::sleep(remaining.min(FOREGROUND_POLL_INTERVAL));
        }
    }

    pub fn play_sine_stream_with_device_hotplug(
        shared_parameters: &Arc<SharedToneParameters>,
    ) -> Result<(), String> {
        let mut carry_state = StreamCarryState {
            phase_radians: 0.0,
            smoothed_linear_gain: linear_gain_from_decibels(shared_parameters.amplitude_decibels()),
            smoothed_frequency_hertz: shared_parameters.frequency_hertz(),
        };
        let mut last_device_fingerprint =
            android_context::output_audio_device_fingerprint().unwrap_or(0);
        let mut opened_once = false;
        let mut pause_was_logged = false;

        while shared_parameters.should_run() {
            if !android_context::activity_window_is_present() {
                if !pause_was_logged {
                    log::info!("Activity window absent; pausing sine output");
                    pause_was_logged = true;
                }
                thread::sleep(FOREGROUND_POLL_INTERVAL);
                continue;
            }
            if pause_was_logged || shared_parameters.output_is_muted() {
                log::info!("Activity window present; resuming sine output");
                pause_was_logged = false;
                shared_parameters.set_output_muted(false);
                if let Some(fingerprint) = android_context::output_audio_device_fingerprint() {
                    last_device_fingerprint = fingerprint;
                }
            }

            let _ = shared_parameters.take_reopen_request();
            match open_and_start_stream(shared_parameters, carry_state) {
                Ok((stream, callback_state_pointer)) => {
                    opened_once = true;
                    log::info!(
                        "AAudio stream started (device fingerprint {last_device_fingerprint})"
                    );
                    let mut next_device_fingerprint_poll_at = Instant::now();
                    while shared_parameters.should_run() {
                        if !android_context::activity_window_is_present() {
                            if !pause_was_logged {
                                log::info!("Activity window absent; pausing sine output");
                                pause_was_logged = true;
                            }
                            // Clear a disconnect reopen so the callback fades instead of stopping.
                            let _ = shared_parameters.take_reopen_request();
                            shared_parameters.set_output_muted(true);
                            sleep_while_running(shared_parameters, OUTPUT_MUTE_FADE_DURATION);
                            break;
                        }
                        if shared_parameters.take_reopen_request() {
                            log::info!("AAudio reopen requested by callback");
                            break;
                        }
                        if Instant::now() >= next_device_fingerprint_poll_at {
                            next_device_fingerprint_poll_at = Instant::now() + DEVICE_POLL_INTERVAL;
                            if let Some(fingerprint) =
                                android_context::output_audio_device_fingerprint()
                            {
                                if fingerprint != last_device_fingerprint {
                                    log::info!(
                                        "Audio output devices changed ({last_device_fingerprint} -> {fingerprint}); reopening stream"
                                    );
                                    last_device_fingerprint = fingerprint;
                                    shared_parameters.request_reopen();
                                    break;
                                }
                            }
                        }
                        thread::sleep(FOREGROUND_POLL_INTERVAL);
                    }
                    carry_state = close_stream(stream, callback_state_pointer);
                    if shared_parameters.output_is_muted() {
                        carry_state.smoothed_linear_gain = 0.0;
                    }
                }
                Err(error_message) => {
                    if !opened_once {
                        return Err(error_message);
                    }
                    log::error!("AAudio reopen failed: {error_message}");
                    sleep_while_running(shared_parameters, REOPEN_COOLDOWN);
                }
            }

            if shared_parameters.should_run() && !shared_parameters.output_is_muted() {
                sleep_while_running(shared_parameters, REOPEN_COOLDOWN);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SharedToneParameters;

    #[test]
    fn output_mute_does_not_change_frequency_or_amplitude() {
        let shared_parameters = SharedToneParameters::new(440.0, -12.0);
        assert!(!shared_parameters.output_is_muted());

        shared_parameters.set_output_muted(true);
        assert!(shared_parameters.output_is_muted());
        assert!((shared_parameters.frequency_hertz() - 440.0).abs() < f32::EPSILON);
        assert!((shared_parameters.amplitude_decibels() - (-12.0)).abs() < f32::EPSILON);

        shared_parameters.set_frequency_hertz(880.0);
        shared_parameters.set_amplitude_decibels(-6.0);
        assert!(shared_parameters.output_is_muted());
        assert!((shared_parameters.frequency_hertz() - 880.0).abs() < f32::EPSILON);
        assert!((shared_parameters.amplitude_decibels() - (-6.0)).abs() < f32::EPSILON);

        shared_parameters.set_output_muted(false);
        assert!(!shared_parameters.output_is_muted());
        assert!((shared_parameters.frequency_hertz() - 880.0).abs() < f32::EPSILON);
        assert!((shared_parameters.amplitude_decibels() - (-6.0)).abs() < f32::EPSILON);
    }
}
