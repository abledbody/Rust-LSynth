mod waveform;
mod channel;

use std::collections::VecDeque;
use crate::channel::Channel;

pub enum Command {
	SetWaveform(usize, usize),
	SetFrequency(usize, f32),
	SetAmplitude(usize, f32),
	SetPanning(usize, f32),
	SetCustomWaveform(usize, [f32; waveform::CUSTOM_WIDTH]),
	
	ForceSetAmplitude(usize, f32),
	ForceSetPanning(usize, f32),
	
	FrequencySlide(usize, f32, f32),
	AmplitudeSlide(usize, f32, f32),
	PanningSlide(usize, f32, f32),
	
	Wait(),
	Request(),
}

pub struct Chip {
	channels: Vec<Channel>,
	samplerate: usize,
	timestep: f32,
	amplitude: f32,
	
	command_buffer: VecDeque<Command>,
	wait_samples: f32,
	tick_samples: f32,
	
	request_callback: Option<fn(&mut Chip)>,
}

impl Chip {
	pub fn new(channel_count: usize, samplerate: usize, amplitude: f32, tick_rate: f32) -> Chip {
		Chip {
			channels: (0..channel_count).map(|_| Channel::new()).collect(),
			samplerate,
			timestep: 1.0/(samplerate as f32),
			amplitude,
			
			command_buffer: VecDeque::new(),
			wait_samples: 0.0,
			tick_samples: Chip::tick_rate_conversion(samplerate, tick_rate),
			request_callback: None,
		}
	}
	
	fn tick_rate_conversion(samplerate: usize, tick_rate: f32) -> f32 {
		(samplerate as f64 * tick_rate as f64) as f32
	}
	
	pub fn set_tick_rate(&mut self, tick_rate: f32) {
		self.tick_samples = Chip::tick_rate_conversion(self.samplerate, tick_rate)
	}
	
	fn get_sample(&self) -> (f32, f32) {
		
		let mut left_sample = 0.0;
		let mut right_sample = 0.0;
		
		for i in 0..self.channels.len() {
			let (channel_left_sample, channel_right_sample) = self.channels[i].sample();
			left_sample += channel_left_sample;
			right_sample += channel_right_sample;
		};
		
		(left_sample * self.amplitude, right_sample * self.amplitude)
	}
	
	pub fn generate(&mut self, buffer: &mut [f32]) {
		let samples = buffer.len();
		
		for sample in (0..samples).step_by(2) {
			if self.wait_samples > 0.0 {
				self.wait_samples -= 1.0;
			}
			else {
				self.execute_commands();
			}
			
			let (left_sample, right_sample) = self.get_sample();
			
			for i in 0..self.channels.len() {
				self.channels[i].advance(self.timestep);
			}
			
			buffer[sample] = left_sample;
			buffer[sample + 1] = right_sample;
		};
	}
	
	pub fn queue_command(&mut self, command: Command) {
		self.command_buffer.push_back(command);
	}
	
	pub fn set_request_callback(&mut self, request_callback: fn(&mut Chip)) {
		self.request_callback = Some(request_callback);
	}
	
	fn tick(&mut self) {
		self.wait_samples += self.tick_samples;
	}
	
	fn execute_commands(&mut self) {
		loop {
			let command = self.command_buffer.pop_front();
			
			match command {
				Some(command) => {
					match command {
						Command::ForceSetAmplitude(channel, value) => {
							self.channels[channel].force_set_amplitude(value);
						}
						
						Command::SetAmplitude(channel, value) => {
							self.channels[channel].set_amplitude(value);
						}
						
						Command::AmplitudeSlide(channel, value, rate) => {
							self.channels[channel].slide_amplitude(value, rate);
						}
						
						Command::SetFrequency(channel, value) => {
							self.channels[channel].set_frequency(value);
						}
						
						Command::FrequencySlide(channel, value, rate) => {
							self.channels[channel].slide_frequency(value, rate);
						}
						
						Command::ForceSetPanning(channel, value) => {
							self.channels[channel].force_set_panning(value);
						}
						
						Command::SetPanning(channel, value) => {
							self.channels[channel].set_panning(value);
						}
						
						Command::PanningSlide(channel, value, rate) => {
							self.channels[channel].slide_panning(value, rate);
						}
						
						Command::SetWaveform(channel, value) => {
							self.channels[channel].set_waveform(value);
						}
						
						Command::SetCustomWaveform(channel, waveform) => {
							self.channels[channel].set_custom_waveform(waveform);
						}
						
						Command::Wait() => {
							self.tick();
							break;
						}
						
						Command::Request() => {
							match self.request_callback {
								Some(func) => func(self),
								None => ()
							}
						}
						
						_ => panic!("Command not implemented"),
					}
				},
				None => {
					self.tick();
					break;
				},
			};
		};
	}
}