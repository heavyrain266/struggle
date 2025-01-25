pub(crate) struct Timer {
	start_time: std::time::Instant,
	last_frame_time: std::time::Instant,
	pub(crate) delta: std::time::Duration,
}

impl Timer {
	pub(crate) fn new() -> Self {
		let now: std::time::Instant = std::time::Instant::now();

		Self {
			start_time: now,
			last_frame_time: now,
			delta: std::time::Duration::ZERO,
		}
	}

	pub(crate) fn update(&mut self) {
		let now: std::time::Instant = std::time::Instant::now();

		self.delta = now - self.last_frame_time;
		self.last_frame_time = now;
	}
}
