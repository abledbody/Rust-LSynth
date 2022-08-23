//! Contains tools for keeping track of the state of individual channels.

use crate::{Command, waveform, errors::*};

/// The time it takes for amplitude and panning changes to occur. This prevents clicks from abrupt changes.
pub const RAMPING_RATE: f32 = 500.0;
/// Used to reduce the wandering of brownian noise. Calculated as `x * (1 - BROWNIAN_LEAK * timestep)`
pub const BROWNIAN_LEAK: f32 = 10000.0;

/// All the parameters needed in order to sample from a channel.
pub(crate) struct ChannelState {
	/// The progress along a repeating waveform on a scale of 0..1. Alternatively, the progress towards generating a new noise sample.
	period: f32,
	/// The current waveform type to use.
	waveform: usize,
	/// The current custom waveform data loaded. Requires the waveform field to be 7 to be generated.
	custom_waveform: waveform::CustomWaveform,
	
	/// The current frequency of the waveform in hertz. Affects the rate at which period is increased.
	frequency: f32,
	/// The current amplitude of the waveform on a scale of 0..1
	amplitude: f32,
	/// The current panning of the waveform on a scale of -1..1. Affects the amplitude of both stereo samples independently.
	panning: f32,
	
	/// The amplitude after being dampened by ramping. This is the actual value the sample uses.
	ramped_amplitude: f32,
	/// The panning after being dampened by ramping. This is the actual value the sample uses.
	ramped_panning: f32,
	
	/// The frequency that the channel is attempting to approach.
	frequency_slide_target: f32,
	/// The amplitude that the channel is attempting to approach.
	amplitude_slide_target: f32,
	/// The panning that the channel is attempting to approach.
	panning_slide_target: f32,
	
	/// The rate at which the frequency approaches ```frequency_slide_target``` in hertz/second
	frequency_rate: f32,
	/// The rate at which the amplitude approaches ```amplitude_slide_target``` in units/second.
	amplitude_rate: f32,
	/// The rate at which the panning approaches ```panning_slide_target``` in units/second.
	panning_rate: f32,
	
	/// The last random value that was generated by the channel. This is what will be sampled until the period elapses.
	noise_sample: f32,
}

impl ChannelState {
	/// Creates a new channel.
	pub(crate) fn new() -> ChannelState {
		ChannelState {
			period: 0.0,
			waveform: 0,
			custom_waveform: [0.0; waveform::CUSTOM_WIDTH],
			
			frequency: 440.0,
			amplitude: 0.0,
			panning: 0.0,
			
			ramped_amplitude: 0.0,
			ramped_panning: 0.0,
			
			frequency_slide_target: 440.0,
			amplitude_slide_target: 0.0,
			panning_slide_target: 0.0,
			
			frequency_rate: 0.0,
			amplitude_rate: 0.0,
			panning_rate: 0.0,
			
			noise_sample: 0.0,
		}
	}
	
	/// Samples the channel in its current state.
	#[no_mangle]
	pub fn sample(&self) -> (f32, f32) {
		let sample_output = match self.waveform {
			0 => waveform::sine(self.period),
			1 => waveform::triangle(self.period),
			2 => waveform::rec_sine(self.period),
			3 => waveform::saw(self.period),
			4 => waveform::square(self.period),
			5 => waveform::pulse(self.period),
			6 => self.noise_sample,
			7 => waveform::custom(self.period, &self.custom_waveform),
			_ => 0.0,
		} * self.ramped_amplitude;
		
		let left_sample = sample_output * (-self.ramped_panning + 1.0).min(1.0);
		let right_sample = sample_output * (self.ramped_panning + 1.0).min(1.0);
		(left_sample, right_sample)
	}
	
	/// Updates the state of the channel by the provided timestep in seconds.
	#[no_mangle]
	pub fn advance(&mut self, step: f32) {
		self.period += self.frequency * step;
		
		if self.waveform == 6 {
			while self.period >= 1.0 {
				self.noise_sample = (self.noise_sample + waveform::noise()) * (1.0 - BROWNIAN_LEAK * step);
				self.period -= 1.0
			}
		}
		
		// This is a really nice way of looping ascending values around 0-1.
		self.period -= self.period.floor();
		
		self.ramped_amplitude = approach(self.ramped_amplitude, self.amplitude, RAMPING_RATE * step);
		self.ramped_panning = approach(self.ramped_panning, self.panning, RAMPING_RATE * step);
		self.frequency = approach(self.frequency, self.frequency_slide_target, self.frequency_rate * step);
		self.amplitude = approach(self.amplitude, self.amplitude_slide_target, self.amplitude_rate * step);
		self.panning = approach(self.panning, self.panning_slide_target, self.panning_rate * step);
	}
	
	/// Executes the provided command immediately.
	#[no_mangle]
	pub fn execute_command(&mut self, command: Command) -> core::result::Result<(), LSynthError> {
		match command {
			Command::ForceSetAmplitude(value) => {
				let value = value.clamp(0_f32, 1_f32);
				self.amplitude = value;
				self.ramped_amplitude = value;
				self.amplitude_slide_target = value;
			}
			
			Command::SetAmplitude(value) => {
				let value = value.clamp(0_f32, 1_f32);
				self.amplitude = value;
				self.amplitude_slide_target = value;
			}
			
			Command::AmplitudeSlide(value, rate) => {
				let value = value.clamp(0_f32, 1_f32);
				self.amplitude_slide_target = value;
				self.amplitude_rate = rate;
			}
			
			Command::SetFrequency(value) => {
				let value = value.max(0_f32);
				self.frequency = value;
				self.frequency_slide_target = value;
			}
			
			Command::FrequencySlide(value, rate) => {
				let value = value.max(0_f32);
				self.frequency_slide_target = value;
				self.frequency_rate = rate;
			}
			
			Command::ForceSetPanning(value) => {
				let value = value.clamp(-1_f32, 1_f32);
				self.panning = value;
				self.ramped_panning = value;
				self.panning_slide_target = value;
			}
			
			Command::SetPanning(value) => {
				let value = value.clamp(-1_f32, 1_f32);
				self.panning = value;
				self.panning_slide_target = value;
			}
			
			Command::PanningSlide(value, rate) => {
				let value = value.clamp(-1_f32, 1_f32);
				self.panning_slide_target = value;
				self.panning_rate = rate;
			}
			
			Command::SetWaveform(value) => {
				if value > 7 {
					return Err(LSynthError::InvalidWaveform(InvalidWaveformError {
						attempted_waveform: value,
					}));
				}
				else {
					self.waveform = value;
				}
			}
			
			Command::SetCustomWaveform(mut waveform) => {
				for value in waveform.iter_mut() {
					*value = value.clamp(-1_f32, 1_f32);
				}
				self.custom_waveform = waveform;
			}
			
			Command::SetPhase(period) => {
				self.period = period % 1.0;
			}
			//_ => panic!("Command not implemented"),
		};
		Ok(())
	}
}

/// Advances value towards target with the provided step.
fn approach(value: f32, target: f32, step: f32) -> f32 {
	let abs_rate = step.abs();
	value + (target - value).min(abs_rate).max(-abs_rate)
}