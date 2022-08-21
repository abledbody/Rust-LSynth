//! Provides C compatible functions for working with this library as a DLL.

use crate::{ChipState, ChipParameters, Command, ChipGenerationData};

/// Initiates a new LSynth chip
#[no_mangle]
pub extern "C" fn ls_init(channel_count: usize, samplerate: usize, amplitude: f32, tick_rate: f32) -> *mut ChipState {
	Box::into_raw(Box::new(ChipState::new(channel_count, ChipParameters::new(samplerate, amplitude, tick_rate))))
}

/// Generates audio with the provided chip.
/// # Safety
/// chip_state must be a valid ChipState generated from the ls_init function.
/// buffer_ptr must point to the first f32 in an array, and buffer_len must be the length of that array.
#[no_mangle]
pub unsafe extern "C" fn ls_generate(chip_state: *mut ChipState, buffer_ptr: *mut f32, buffer_len: usize, buffer_start: usize) -> ChipGenerationData {
	let chip_state = &mut *chip_state;
	let buffer = std::slice::from_raw_parts_mut(buffer_ptr, buffer_len);
	
	chip_state.generate(&mut buffer[buffer_start..]).unwrap()
}

/// Inserts a command into the provided chip_state
/// # Safety
/// chip_state must be a valid ChipState generated from the ls_init function.
#[no_mangle]
pub unsafe extern "C" fn ls_send_command(chip_state: *mut ChipState, command: Command, channel: usize) {
	let chip_state = &mut *chip_state;
	
	let _ = chip_state.send_command(command, channel);
}

/// Returns the number of samples that are in a single tick.
/// # Safety
/// chip_state must be a valid ChipState generated from the ls_init function.
#[no_mangle]
pub unsafe extern "C" fn ls_get_tick_frames(chip_state: *mut ChipState) -> f32 {
	let chip_state = & *chip_state;
	chip_state.parameters.get_tick_frames()
}

/// Sends a SetWaveform command to the given channel.
/// # Safety
/// chip_state must be a valid ChipState generated from the ls_init function.
#[no_mangle]
pub unsafe extern "C" fn ls_set_waveform(chip_state: *mut ChipState, channel: usize, waveform: usize) {
	let chip_state = &mut *chip_state;
	let _ = chip_state.send_command(Command::SetWaveform(waveform), channel);
}

/// Sends a SetFrequency command to the given channel.
/// # Safety
/// chip_state must be a valid ChipState generated from the ls_init function.
#[no_mangle]
pub unsafe extern "C" fn ls_set_frequency(chip_state: *mut ChipState, channel: usize, frequency: f32) {
	let chip_state = &mut *chip_state;
	let _ = chip_state.send_command(Command::SetFrequency(frequency), channel);
}

/// Sends a SetAmplitude command to the given channel.
/// # Safety
/// chip_state must be a valid ChipState generated from the ls_init function.
#[no_mangle]
pub unsafe extern "C" fn ls_set_amplitude(chip_state: *mut ChipState, channel: usize, amplitude: f32) {
	let chip_state = &mut *chip_state;
	let _ = chip_state.send_command(Command::SetAmplitude(amplitude), channel);
}

/// Sends a SetPanning command to the given channel.
/// # Safety
/// chip_state must be a valid ChipState generated from the ls_init function.
#[no_mangle]
pub unsafe extern "C" fn ls_set_panning(chip_state: *mut ChipState, channel: usize, panning: f32) {
	let chip_state = &mut *chip_state;
	let _ = chip_state.send_command(Command::SetPanning(panning), channel);
}

/// Sends a SetCustomWaveform command to the given channel.
/// # Safety
/// chip_state must be a valid ChipState generated from the ls_init function.
/// waveform_ptr must point to the first f32 in an array, and waveform_len must be the length of that array.
#[no_mangle]
pub unsafe extern "C" fn ls_set_custom_waveform(chip_state: *mut ChipState, channel: usize, waveform_ptr: *mut f32, waveform_len: usize) {
	let chip_state = &mut *chip_state;
	
	let mut waveform = [0.0; crate::waveform::CUSTOM_WIDTH];
	waveform.clone_from_slice(std::slice::from_raw_parts(waveform_ptr, waveform_len));
	let _ = chip_state.send_command(Command::SetCustomWaveform(waveform), channel);
}

/// Sends a SetPhase command to the given channel.
/// # Safety
/// chip_state must be a valid ChipState generated from the ls_init function.
#[no_mangle]
pub unsafe extern "C" fn ls_set_phase(chip_state: *mut ChipState, channel: usize, phase: f32) {
	let chip_state = &mut *chip_state;
	let _ = chip_state.send_command(Command::SetPhase(phase), channel);
}

/// Sends a ForceSetAmplitude command to the given channel.
/// # Safety
/// chip_state must be a valid ChipState generated from the ls_init function.
#[no_mangle]
pub unsafe extern "C" fn ls_force_set_amplitude(chip_state: *mut ChipState, channel: usize, amplitude: f32) {
	let chip_state = &mut *chip_state;
	let _ = chip_state.send_command(Command::ForceSetAmplitude(amplitude), channel);
}

/// Sends a ForceSetPanning command to the given channel.
/// # Safety
/// chip_state must be a valid ChipState generated from the ls_init function.
#[no_mangle]
pub unsafe extern "C" fn ls_force_set_panning(chip_state: *mut ChipState, channel: usize, panning: f32) {
	let chip_state = &mut *chip_state;
	let _ = chip_state.send_command(Command::ForceSetPanning(panning), channel);
}

/// Sends a FrequencySlide command to the given channel.
/// # Safety
/// chip_state must be a valid ChipState generated from the ls_init function.
#[no_mangle]
pub unsafe extern "C" fn ls_frequency_slide(chip_state: *mut ChipState, channel: usize, frequency: f32, rate: f32) {
	let chip_state = &mut *chip_state;
	let _ = chip_state.send_command(Command::FrequencySlide(frequency, rate), channel);
}

/// Sends a AmplitudeSlide command to the given channel.
/// # Safety
/// chip_state must be a valid ChipState generated from the ls_init function.
#[no_mangle]
pub unsafe extern "C" fn ls_amplitude_slide(chip_state: *mut ChipState, channel: usize, amplitude: f32, rate: f32) {
	let chip_state = &mut *chip_state;
	let _ = chip_state.send_command(Command::AmplitudeSlide(amplitude, rate), channel);
}

/// Sends a PanningSlide command to the given channel.
/// # Safety
/// chip_state must be a valid ChipState generated from the ls_init function.
#[no_mangle]
pub unsafe extern "C" fn ls_panning_slide(chip_state: *mut ChipState, channel: usize, panning: f32, rate: f32) {
	let chip_state = &mut *chip_state;
	let _ = chip_state.send_command(Command::PanningSlide(panning, rate), channel);
}