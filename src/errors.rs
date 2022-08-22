//! Contains the error types that LSynth could return

use std::fmt::Debug;

/// Errors that are returned when LSynth is given invalid instructions.
pub enum LSynthError {
	/// Attempted to send a command to set the channel to a waveform that does not exist.
	InvalidWaveform(InvalidWaveformError),
	/// Attempted to send a command to a channel that does not exist.
	InvalidChannel(InvalidChannelError),
	/// Attempted to fill a buffer with an odd number of samples.
	UnevenBufferSlice(UnevenBufferSliceError),
}

impl Debug for LSynthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidWaveform(err) => write!(f, "{:?}", err),
            Self::InvalidChannel(err) => write!(f, "{:?}", err),
            Self::UnevenBufferSlice(err) => write!(f, "{:?}", err),
        }
    }
}

/// Occurs when attempting to send a command to set the channel to a waveform that does not exist.
pub struct InvalidWaveformError {
	/// The number that was attempted to be used as a waveform index.
	pub attempted_waveform: usize,
}

impl Debug for InvalidWaveformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Attempted to set LSynth channel to invalid waveform: {}", self.attempted_waveform)
    }
}

/// Occurs when attempting to send a command to a channel that does not exist.
pub struct InvalidChannelError {
	/// The channel that a command was attempted to be sent to.
	pub attempted_channel: usize,
	/// How many channels the chip actually has.
	pub max_channels_of_chip: usize,
}

impl Debug for InvalidChannelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unable to send command to channel {}. Chip only has {} channels.", self.attempted_channel, self.max_channels_of_chip)
    }
}

/// Occurs when attempting to fill a buffer with an odd number of samples.
pub struct UnevenBufferSliceError {
	/// The length of the slice.
	pub slice_length: usize,
}

impl Debug for UnevenBufferSliceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Provided slice length of {} is an odd number. Cannot generate stereo audio.", self.slice_length)
    }
}