use crate::waveform;

pub const RAMPING_TIME: f32 = 0.002;

pub(crate) struct Channel {
	period: f32,
	waveform: usize,
	custom_waveform: [f32; waveform::CUSTOM_WIDTH],
	
	frequency: f32,
	amplitude: f32,
	panning: f32,
	
	frequency_slide_target: f32,
	amplitude_slide_target: f32,
	panning_slide_target: f32,
	
	frequency_rate: f32,
	amplitude_rate: f32,
	panning_rate: f32,
	
	noise_sample: f32,
}

impl Channel {
	pub fn new() -> Channel {
		Channel {
			period: 0.0,
			waveform: 0,
			custom_waveform: [0.0; 16],
			
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
	
	pub fn advance(&mut self, step: f32) {
		self.period += self.frequency * step;
		
		if self.period >= 1.0 {
			self.noise_sample = waveform::noise();
		}
		
		// This is a really nice way of looping ascending values around 0-1.
		self.period = self.period - self.period.floor();
		
		self.frequency += slide(self.frequency, self.frequency_slide_target, self.frequency_rate * step);
		self.amplitude += slide(self.amplitude, self.amplitude_slide_target, self.amplitude_rate * step);
		self.panning += slide(self.panning, self.panning_slide_target, self.panning_rate * step);
	}
	
	pub fn force_set_amplitude(&mut self, value: f32) {
		self.amplitude = value;
		self.amplitude_slide_target = value;
	}
	
	pub fn set_amplitude(&mut self, value: f32) {
		let slide_rate = (self.amplitude - value) / RAMPING_TIME;
		self.slide_amplitude(value, slide_rate);
	}
	
	pub fn slide_amplitude(&mut self, value: f32, rate: f32) {
		self.amplitude_slide_target = value;
		self.amplitude_rate = rate;
	}
	
	pub fn set_frequency(&mut self, value: f32) {
		self.frequency = value;
		self.frequency_slide_target = value;
	}
	
	pub fn slide_frequency(&mut self, value: f32, rate: f32) {
		self.frequency_slide_target = value;
		self.frequency_rate = rate;
	}
	
	pub fn force_set_panning(&mut self, value: f32) {
		self.panning = value;
		self.panning_slide_target = value;
	}
	
	pub fn set_panning(&mut self, value: f32) {
		let slide_rate = (self.panning - value) / RAMPING_TIME;
		self.slide_panning(value, slide_rate);
	}
	
	pub fn slide_panning(&mut self, value: f32, rate: f32) {
		self.panning_slide_target = value;
		self.panning_rate = rate;
	}
	
	pub fn set_waveform(&mut self, value: usize) {
		self.waveform = value;
	}
}

fn slide(value: f32, target: f32, rate: f32) -> f32 {
	let abs_rate = rate.abs();
	(target - value).min(abs_rate).max(-abs_rate)
}