//! Contains tools for keeping track of the state of individual channels.

use crate::waveform::{self, CustomWaveform};

/// The time it takes for amplitude and panning changes to occur. This prevents clicks from abrupt changes.
pub const RAMPING_TIME: f32 = 0.002;

#[derive(Clone)]
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
	pub fn new() -> ChannelState {
		ChannelState {
			period: 0.0,
			waveform: 0,
			custom_waveform: [0.0; waveform::CUSTOM_WIDTH],
			
			frequency: 440.0,
			amplitude: 0.0,
			panning: 0.0,
			
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
		} * self.amplitude;
		
		let left_sample = sample_output * (-self.panning + 1.0).min(1.0);
		let right_sample = sample_output * (self.panning + 1.0).min(1.0);
		(left_sample, right_sample)
	}
	
	/// Updates the state of the channel by the provided timestep in seconds.
	pub fn advance(&mut self, step: f32) {
		self.period += self.frequency * step;
		
		if self.period >= 1.0 {
			self.noise_sample = waveform::noise();
		}
		
		// This is a really nice way of looping ascending values around 0-1.
		self.period -= self.period.floor();
		
		self.frequency = approach(self.frequency, self.frequency_slide_target, self.frequency_rate * step);
		self.amplitude = approach(self.amplitude, self.amplitude_slide_target, self.amplitude_rate * step);
		self.panning = approach(self.panning, self.panning_slide_target, self.panning_rate * step);
	}
	
	/// Immediately sets the amplitude of the channel to the provided value.
	pub fn force_set_amplitude(&mut self, value: f32) {
		self.amplitude = value;
		self.amplitude_slide_target = value;
	}
	
	/// Sets the amplitude of the channel to the provided value with ramping.
	pub fn set_amplitude(&mut self, value: f32) {
		let slide_rate = (self.amplitude - value) / RAMPING_TIME;
		self.slide_amplitude(value, slide_rate);
	}
	
	/// Initiates a slide from the current amplitude to the target amplitude with the provided rate.
	pub fn slide_amplitude(&mut self, value: f32, rate: f32) {
		self.amplitude_slide_target = value;
		self.amplitude_rate = rate;
	}
	
	/// Immediately sets the frequency of the channel to the provided value.
	pub fn set_frequency(&mut self, value: f32) {
		self.frequency = value;
		self.frequency_slide_target = value;
	}
	
	/// Initiates a slide from the current frequency to the target frequency with the provided rate.
	pub fn slide_frequency(&mut self, value: f32, rate: f32) {
		self.frequency_slide_target = value;
		self.frequency_rate = rate;
	}
	
	/// Immediately sets the panning of the channel to the provided value.
	pub fn force_set_panning(&mut self, value: f32) {
		self.panning = value;
		self.panning_slide_target = value;
	}
	
	/// Sets the panning of the channel to the provided value with ramping.
	pub fn set_panning(&mut self, value: f32) {
		let slide_rate = (self.panning - value) / RAMPING_TIME;
		self.slide_panning(value, slide_rate);
	}
	
	/// Initiates a slide from the current panning to the target panning with the provided rate.
	pub fn slide_panning(&mut self, value: f32, rate: f32) {
		self.panning_slide_target = value;
		self.panning_rate = rate;
	}
	
	/// Sets the type of waveform that this channel will be generating samples from.
	pub fn set_waveform(&mut self, value: usize) {
		self.waveform = value;
	}
	
	/// Updates the channels current custom waveform. This will not be played unless the current waveform is 7.
	pub fn set_custom_waveform(&mut self, waveform: CustomWaveform) {
		self.custom_waveform = waveform;
	}
}

/// Advances value towards target with the provided step.
fn approach(value: f32, target: f32, step: f32) -> f32 {
	let abs_rate = step.abs();
	value + (target - value).min(abs_rate).max(-abs_rate)
}