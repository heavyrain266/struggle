//! Timer
//!
//! A module used to track elapsed time between frames.

pub(crate) struct Timer {
	/// The time at which the last frame was updated.
	last_frame: std::time::Instant,

	/// The time elapsed since the last frame update.
	pub(crate) delta: std::time::Duration,
}

impl Timer {
	/// Constructor
	///
	/// A method that creates new instance of [`Timer`].
	pub(crate) fn new() -> Self {
		let now: std::time::Instant = std::time::Instant::now();

		Self {
			last_frame: now,
			delta: std::time::Duration::ZERO,
		}
	}

	/// Update
	///
	/// A method that calculates the time elapsed since the last frame update.
	pub(crate) fn update(&mut self) {
		let now: std::time::Instant = std::time::Instant::now();

		self.delta = now - self.last_frame;
		self.last_frame = now;
	}
}
