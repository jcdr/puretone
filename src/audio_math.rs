pub const MINIMUM_FREQUENCY_HERTZ: f32 = 50.0;
pub const MAXIMUM_FREQUENCY_HERTZ: f32 = 4000.0;
pub const MINIMUM_AMPLITUDE_DECIBELS: f32 = -80.0;
pub const MAXIMUM_AMPLITUDE_DECIBELS: f32 = 0.0;
pub const AUDIO_SAMPLE_RATE_HERTZ: u32 = 48_000;
pub const TWO_PI: f32 = std::f32::consts::TAU;
pub const PARAMETER_SMOOTHING_TIME_CONSTANT_SECONDS: f32 = 0.010;

pub fn frequency_hertz_from_log_normalized(normalized_position: f32) -> f32 {
    let clamped = normalized_position.clamp(0.0, 1.0);
    let frequency_ratio = MAXIMUM_FREQUENCY_HERTZ / MINIMUM_FREQUENCY_HERTZ;
    MINIMUM_FREQUENCY_HERTZ * frequency_ratio.powf(clamped)
}

pub fn log_normalized_from_frequency_hertz(frequency_hertz: f32) -> f32 {
    let clamped = frequency_hertz.clamp(MINIMUM_FREQUENCY_HERTZ, MAXIMUM_FREQUENCY_HERTZ);
    let frequency_ratio = MAXIMUM_FREQUENCY_HERTZ / MINIMUM_FREQUENCY_HERTZ;
    (clamped / MINIMUM_FREQUENCY_HERTZ).log(frequency_ratio)
}

pub fn linear_gain_from_decibels(amplitude_decibels: f32) -> f32 {
    let clamped = amplitude_decibels.clamp(MINIMUM_AMPLITUDE_DECIBELS, MAXIMUM_AMPLITUDE_DECIBELS);
    10.0_f32.powf(clamped / 20.0)
}

pub fn sine_sample_from_phase(phase_radians: f32) -> f32 {
    phase_radians.sin()
}

pub fn advance_phase_radians(
    phase_radians: f32,
    frequency_hertz: f32,
    sample_rate_hertz: f32,
) -> f32 {
    let phase_increment = TWO_PI * frequency_hertz / sample_rate_hertz;
    let advanced = phase_radians + phase_increment;
    if advanced >= TWO_PI {
        advanced % TWO_PI
    } else {
        advanced
    }
}

pub fn render_sine_sample(
    phase_radians: f32,
    frequency_hertz: f32,
    amplitude_decibels: f32,
    sample_rate_hertz: f32,
) -> (f32, f32) {
    let sample = sine_sample_from_phase(phase_radians) * linear_gain_from_decibels(amplitude_decibels);
    let next_phase = advance_phase_radians(phase_radians, frequency_hertz, sample_rate_hertz);
    (sample, next_phase)
}

pub fn one_pole_smoothing_coefficient(
    time_constant_seconds: f32,
    sample_rate_hertz: f32,
) -> f32 {
    let safe_time_constant = time_constant_seconds.max(1.0e-4);
    let safe_sample_rate = sample_rate_hertz.max(1.0);
    1.0 - (-1.0 / (safe_time_constant * safe_sample_rate)).exp()
}

pub fn smooth_toward(current_value: f32, target_value: f32, coefficient: f32) -> f32 {
    let clamped_coefficient = coefficient.clamp(0.0, 1.0);
    current_value + (target_value - current_value) * clamped_coefficient
}

pub fn parameter_smoothing_coefficient(sample_rate_hertz: f32) -> f32 {
    one_pole_smoothing_coefficient(
        PARAMETER_SMOOTHING_TIME_CONSTANT_SECONDS,
        sample_rate_hertz,
    )
}

pub fn gain_smoothing_coefficient(sample_rate_hertz: f32) -> f32 {
    parameter_smoothing_coefficient(sample_rate_hertz)
}

pub fn frequency_smoothing_coefficient(sample_rate_hertz: f32) -> f32 {
    parameter_smoothing_coefficient(sample_rate_hertz)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_frequency_maps_endpoints() {
        let minimum = frequency_hertz_from_log_normalized(0.0);
        let maximum = frequency_hertz_from_log_normalized(1.0);
        assert!((minimum - MINIMUM_FREQUENCY_HERTZ).abs() < 1e-3);
        assert!((maximum - MAXIMUM_FREQUENCY_HERTZ).abs() < 1e-2);
    }

    #[test]
    fn log_frequency_midpoint_is_geometric_mean() {
        let midpoint_frequency = frequency_hertz_from_log_normalized(0.5);
        let expected = (MINIMUM_FREQUENCY_HERTZ * MAXIMUM_FREQUENCY_HERTZ).sqrt();
        assert!((midpoint_frequency - expected).abs() < 1e-2);
    }

    #[test]
    fn log_frequency_round_trip() {
        for frequency in [50.0_f32, 100.0, 440.0, 1000.0, 4000.0] {
            let normalized = log_normalized_from_frequency_hertz(frequency);
            let restored = frequency_hertz_from_log_normalized(normalized);
            assert!((restored - frequency).abs() < 1e-2, "failed at {frequency}");
        }
    }

    #[test]
    fn decibel_zero_is_unity_gain() {
        let gain = linear_gain_from_decibels(0.0);
        assert!((gain - 1.0).abs() < 1e-6);
    }

    #[test]
    fn decibel_minus_twenty_is_tenth_gain() {
        let gain = linear_gain_from_decibels(-20.0);
        assert!((gain - 0.1).abs() < 1e-5);
    }

    #[test]
    fn decibel_minus_eighty_is_very_small() {
        let gain = linear_gain_from_decibels(-80.0);
        assert!(gain > 0.0);
        assert!(gain < 0.0002);
        assert!((gain - 1e-4).abs() < 1e-6);
    }

    #[test]
    fn sine_at_zero_phase_is_zero() {
        let sample = sine_sample_from_phase(0.0);
        assert!(sample.abs() < 1e-6);
    }

    #[test]
    fn sine_at_quarter_turn_is_one() {
        let sample = sine_sample_from_phase(std::f32::consts::FRAC_PI_2);
        assert!((sample - 1.0).abs() < 1e-5);
    }

    #[test]
    fn render_sine_sample_applies_gain_and_advances_phase() {
        let frequency = 480.0_f32;
        let sample_rate = AUDIO_SAMPLE_RATE_HERTZ as f32;
        let (sample, next_phase) = render_sine_sample(0.0, frequency, -6.0, sample_rate);
        assert!(sample.abs() < 1e-5);
        let expected_increment = TWO_PI * frequency / sample_rate;
        assert!((next_phase - expected_increment).abs() < 1e-5);
        let expected_gain = linear_gain_from_decibels(-6.0);
        let (peak_sample, _) =
            render_sine_sample(std::f32::consts::FRAC_PI_2, frequency, -6.0, sample_rate);
        assert!((peak_sample - expected_gain).abs() < 1e-5);
    }

    #[test]
    fn smooth_toward_moves_partway_to_target() {
        let smoothed = smooth_toward(0.0, 1.0, 0.25);
        assert!((smoothed - 0.25).abs() < 1e-6);
    }

    #[test]
    fn smooth_toward_reaches_target_with_full_coefficient() {
        let smoothed = smooth_toward(0.1, 0.9, 1.0);
        assert!((smoothed - 0.9).abs() < 1e-6);
    }

    #[test]
    fn parameter_smoothing_coefficient_is_between_zero_and_one() {
        let coefficient = parameter_smoothing_coefficient(AUDIO_SAMPLE_RATE_HERTZ as f32);
        assert!(coefficient > 0.0);
        assert!(coefficient < 0.1);
        assert!((coefficient - gain_smoothing_coefficient(AUDIO_SAMPLE_RATE_HERTZ as f32)).abs() < 1e-9);
        assert!(
            (coefficient - frequency_smoothing_coefficient(AUDIO_SAMPLE_RATE_HERTZ as f32)).abs()
                < 1e-9
        );
    }

    #[test]
    fn parameter_smoothing_reduces_step_size_over_short_horizon() {
        let sample_rate = AUDIO_SAMPLE_RATE_HERTZ as f32;
        let coefficient = parameter_smoothing_coefficient(sample_rate);
        let target_gain = linear_gain_from_decibels(0.0);
        let mut current_gain = linear_gain_from_decibels(-40.0);
        let initial_gain_error = (target_gain - current_gain).abs();
        current_gain = smooth_toward(current_gain, target_gain, coefficient);
        assert!((target_gain - current_gain).abs() < initial_gain_error);
        let samples_for_five_time_constants =
            (5.0 * PARAMETER_SMOOTHING_TIME_CONSTANT_SECONDS * sample_rate) as usize;
        for _ in 0..samples_for_five_time_constants {
            current_gain = smooth_toward(current_gain, target_gain, coefficient);
        }
        assert!((current_gain - target_gain).abs() < 0.02 * initial_gain_error.max(1.0));

        let target_frequency = MAXIMUM_FREQUENCY_HERTZ;
        let mut current_frequency = MINIMUM_FREQUENCY_HERTZ;
        let initial_frequency_error = (target_frequency - current_frequency).abs();
        for _ in 0..samples_for_five_time_constants {
            current_frequency =
                smooth_toward(current_frequency, target_frequency, coefficient);
        }
        assert!(
            (current_frequency - target_frequency).abs()
                < 0.02 * initial_frequency_error.max(1.0)
        );
    }
}
