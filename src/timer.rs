//! Timer
//!
//! A module used to track elapsed time between frames.

pub struct Timer {
	last_frame: std::time::Instant,
	pub delta: std::time::Duration,
}

impl Timer {
	pub fn new() -> Self {
		let now: std::time::Instant = std::time::Instant::now();

		Self {
			last_frame: now,
			delta: std::time::Duration::ZERO,
		}
	}

	pub fn update(&mut self) {
		let now: std::time::Instant = std::time::Instant::now();

		self.delta = now - self.last_frame;
		self.last_frame = now;
	}
}
