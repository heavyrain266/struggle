//! Struggle
//!
//! Some graphics tests.

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_return)]
#![allow(clippy::cast_possible_truncation)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod hal;
mod timer;

//#[doc(hidden)]
//#[global_allocator]
//static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[doc(hidden)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
	let event_loop: winit::event_loop::EventLoop<()> = winit::event_loop::EventLoop::new()?;
	let mut struggle: app::Struggle = app::Struggle::default();

	event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
	event_loop.run_app(&mut struggle)?;

	return Ok(());
}
