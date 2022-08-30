#![deny(clippy::missing_docs_in_private_items)]

//! This library is for generating LSynth audio streams.
//! 
//! Here is an example of the basic setup of the LSynth chip.
//! ```
//! use lsynth::*;
//! 
//! let mut chip = ChipState::new(4, ChipParameters::new(44_100, 0.5, 120.0));
//! 
//! chip.send_command(Command::SetAmplitude(0.0), 0);
//! chip.send_command(Command::SetFrequency(110.0), 0);
//! 
//! let mut frequency = 110.0;
//! let mut beat = 0;
//! 
//! let mut request_callback = move |chip: &mut ChipState| {
//!     beat += 1;
//!     while beat >= 4 {
//!         frequency += 110.0;
//! 
//!         chip.send_command(Command::SetFrequency(frequency), 0);
//!         beat -= 4;
//!     }
//! };
//! 
//! let mut audio_sample_request = move |buffer: &mut [f32]| {
//!     let mut sample_index = 0;
//!     
//!     while sample_index < buffer.len() {
//!         let generated_data = chip.generate(&mut buffer[sample_index..]).unwrap();
//!         sample_index += generated_data.generated;
//!         
//!         assert!(generated_data.generated != 0);
//!     
//!         if generated_data.remaining_samples == 0 { request_callback(&mut chip); }
//!     }
//! };
//! #
//! # let mut audio_stream = [0.0; 512];
//! # audio_sample_request(&mut audio_stream);
//! ```

pub mod waveform;
mod channel;
pub mod errors;
pub mod c_compatible;

use channel::ChannelState;
use errors::{InvalidChannelError, LSynthError, UnevenBufferSliceError};
use serde::{Serialize, Deserialize};

/// The different types of commands that can be sent to channels.
#[derive(Clone)]
#[derive(Serialize, Deserialize)]
#[repr(C)]
pub enum Command {
	/// An instruction to set the waveform of the channel.
	///
	/// | Index | Type              |
	/// |---|----------------|
	/// | 0 | Sine           |
	/// | 1 | Triangle       |
	/// | 2 | Rectified Sine |
	/// | 3 | Saw            |
	/// | 4 | Square         |
	/// | 5 | Pulse          |
	/// | 6 | Noise          |
	/// | 7 | Custom         |
	SetWaveform(usize),
	/// An instruction to set the frequency of the channel in hertz.
	SetFrequency(f32),
	/// An instruction to set the amplitude of the channel on a scale of 0..1
	SetAmplitude(f32),
	/// An instruction to set the panning of the channel on a scale of -1..1
	SetPanning(f32),
	/// An instruction to change the custom waveform stored in the channel.
	SetCustomWaveform(waveform::CustomWaveform),
	/// An instruction to set the phase of a waveform directly.
	SetPhase(f32),
	
	/// An instruction to change the amplitude of the channel instantly, instead of softly.
	ForceSetAmplitude(f32),
	/// An instruction to change the panning of the channel instantly, instead of softly.
	ForceSetPanning(f32),
	
	/// An instruction to gradually change the frequency of the channel from its current state to a target state with the specified rate of change.
	FrequencySlide(f32, f32),
	/// An instruction to gradually change the amplitude of the channel from its current state to a target state with the specified rate of change.
	AmplitudeSlide(f32, f32),
	/// An instruction to gradually change the panning of the channel from its current state to a target state with the specified rate of change.
	PanningSlide(f32, f32),
}

/// The current state of the LSynth chip.
pub struct ChipState {
	/// The states of all the channels currently operated by LSynth.
	channels: Vec<ChannelState>,
	/// Details how this chip is intended to operate.
	pub parameters: ChipParameters,
	/// How many frames are left in this tick.
	remaining_frames: f32,
}

/// Parameters detailing how an LSynth chip is intended to operate.
#[derive(Serialize, Deserialize)]
pub struct ChipParameters {
	/// The samplerate in hertz.
	samplerate: usize,
	/// Seconds per sample
	timestep: f32,
	/// The global amplitude of this chip on a scale of 0..1. Affects all channels.
	amplitude: f32,
	/// The number of samples there are in a single tick.
	tick_rate: f32,
	/// The number of samples there are in a single tick.
	tick_frames: f32,
}

/// Data returned by the generate function of ChipState.
#[repr(C)]
pub struct ChipGenerationData {
	/// How many samples were generated.
	pub generated: usize,
	/// How many samples were left in the tick when generation stopped.
	pub remaining_samples: usize,
}

impl ChipParameters {
	/// Creates a new set of chip parameters. Tick rate is ticks per second.
	pub fn new(samplerate: usize, amplitude: f32, tick_rate: f32) -> ChipParameters {
		ChipParameters {
			samplerate,
			timestep: 1.0/(samplerate as f32),
			amplitude,
			tick_rate,
			tick_frames: samplerate as f32 / tick_rate
		}
	}
	
	/// Converts the from ticks per second to samples per tick.
	fn update_tick_frames(&mut self) {
	 	self.tick_frames = self.samplerate as f32 / self.tick_rate
	}
	
	/// Sets the samplerate of the chip in hertz.
	pub fn set_sample_rate(&mut self, samplerate: usize) {
		self.samplerate = samplerate;
		self.timestep = 1.0/(samplerate as f32);
		self.update_tick_frames();
	}
	
	/// Sets the tick rate of the chip in hertz.
	pub fn set_tick_rate(&mut self, tick_rate: f32) {
		self.tick_rate = tick_rate;
		self.update_tick_frames();
	}
	
	/// Returns the number of samples in a single tick.
	pub fn get_tick_frames(&self) -> f32 {
	 	self.tick_frames
	}
}

impl ChipState {
	/// Creates a new LSynth chip.
	pub fn new(channel_count: usize, parameters: ChipParameters) -> ChipState {
		ChipState {
			channels: (0..channel_count).map(|_| ChannelState::new()).collect(),
			parameters,
			remaining_frames: 0.0,
		}
	}
	
	/// Writes a tick worth of interlaced stereo samples generated by the chip to the start of the provided slice,
	/// then returns a struct containing information about how many samples it generated,
	/// and how many samples still need to be generated to complete a tick.
	/// 
	/// If the number of remaining samples is anything but zero, then the tick was not completed.
	/// Commands can still be sent at this point, but they will occur in between ticks.
	pub fn generate(&mut self, buffer: &mut [f32]) -> Result<ChipGenerationData, LSynthError> {
		use rayon::prelude::*;
		
		if buffer.len() % 2 != 0 {
			return Err(LSynthError::UnevenBufferSlice(UnevenBufferSliceError{slice_length: buffer.len()}));
		}
		
		// Don't want to have to borrow this.
		let timestep = self.parameters.timestep;
		
		if self.remaining_frames < 1.0 {
			self.remaining_frames += self.parameters.get_tick_frames();
		}
		
		let frames_to_generate = (self.remaining_frames.floor() as usize).min(buffer.len() / 2);
		
		// Generate from each channel on its own thread.
		let frame_vecs: Vec<Vec<(f32, f32)>> = self.channels.par_iter_mut()
			.map(|channel| {
				let mut frames = vec![(0.0, 0.0); frames_to_generate];
				for value in frames.iter_mut() {
					*value = channel.sample();
					channel.advance(timestep);
				}
				frames
			})
			.collect();
		
		// Iterating over frame_vecs would give us access to one channel at a time, which is not helpful,
		// so instead we're iterating over the slice of the buffer we intend to fill.
		for (i, frame) in buffer.chunks_mut(2).enumerate() {
			if i >= frames_to_generate { break; }
			frame[0] = 0.0;
			frame[1] = 0.0;
			
			for channel in frame_vecs.iter() {
				let (l, r) = channel[i];
				frame[0] += l * self.parameters.amplitude;
				frame[1] += r * self.parameters.amplitude;
			}
		}
		
		// Adds only the fractional part of tick_frames.
		self.remaining_frames -= frames_to_generate as f32;
		
		Ok(ChipGenerationData {generated: frames_to_generate * 2, remaining_samples: (self.remaining_frames.floor() as usize) * 2})
	}
	
	/// Executes a command on the given channel.
	pub fn send_command(&mut self, command: Command, channel: usize) -> Result<(), LSynthError> {
		if channel < self.channels.len() {
			self.channels[channel].execute_command(command)?;
			Ok(())
		}
		else {
			Err(LSynthError::InvalidChannel(InvalidChannelError {
				max_channels_of_chip: self.channels.len(),
				attempted_channel: channel,
			}))
		}
	}
}