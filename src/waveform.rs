use rand;

pub const CUSTOM_WIDTH: usize = 16;


pub fn sine(period: f32) -> f32 {
	f32::sin(period * std::f32::consts::TAU)
}

pub fn triangle(period: f32) -> f32 {
	let period = 
		if period < 0.75 {period + 0.25} else {period - 0.75};
	
	if period < 0.5 {
		period * 4.0 - 1.0
	}
	else {
		1.0 - (period - 0.5) * 4.0
	}
}

pub fn rec_sine(period: f32) -> f32 {
	if period < 0.5 {
		f32::sin(period * std::f32::consts::TAU)
	}
	else {
		0.0
	}
}

pub fn saw(period: f32) -> f32 {
	let sample = period * 2.0;
	sample - sample.floor() * 2.0
}

pub fn square(period: f32) -> f32 {
	if period < 0.5 {
		1.0
	}
	else {
		-1.0
	}
}

pub fn pulse(period: f32) -> f32 {
	if period < 0.25 {
		1.0
	}
	else {
		-1.0
	}
}

pub fn noise() -> f32 {
	rand::random::<f32>() * 2.0 - 1.0
}

pub fn custom(period: f32, data: &[f32; CUSTOM_WIDTH]) -> f32{
	let index = (period * CUSTOM_WIDTH as f32).floor() as usize;
	data[index]
}