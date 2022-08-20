//! Contains the formulas for generating all the different types of waveforms. All generated samples are between -1 and 1, and the provided periods are expected to be between 0 and 1.

/// The number of samples in a custom waveform.
pub const CUSTOM_WIDTH: usize = 32;

/// Custom waveforms only need to contain an array of data. This is a convenience type for arrays that follow the required pattern.
pub type CustomWaveform = [f32; CUSTOM_WIDTH];

/// Generates a sinewave
pub(crate) fn sine(period: f32) -> f32 {
	f32::sin(period * std::f32::consts::TAU)
}

/// Generates a trianglewave
pub(crate) fn triangle(period: f32) -> f32 {
	-(period - 0.5).abs() * 4.0 + 1.0
}

/// Generates a sinewave where the negative values have been truncated. Scaled to generate values between -1 and 1.
pub(crate) fn rec_sine(period: f32) -> f32 {
	if period < 0.5 {
		f32::sin(period * std::f32::consts::TAU) * 2.0 - 1.0
	}
	else {
		-1.0
	}
}

/// Generates a sawwave.
pub(crate) fn saw(period: f32) -> f32 {
	period * 2.0 - 1.0
}

/// Generates a pulse wave with a duty of 50%.
pub(crate) fn square(period: f32) -> f32 {
	if period < 0.5 {1.0}
	else {-1.0}
}

/// Generates a pulse wave with a duty of 25%.
pub(crate) fn pulse(period: f32) -> f32 {
	if period < 0.25 {1.0}
	else {-1.0}
}

/// Generates a random number between -1 and 1.
pub(crate) fn noise() -> f32 {
	rand::random::<f32>() * 2.0 - 1.0
}

/// Samples a custom waveform at the given point in the period.
pub(crate) fn custom(period: f32, data: &CustomWaveform) -> f32{
	let index = (period * CUSTOM_WIDTH as f32).floor() as usize;
	data[index]
}