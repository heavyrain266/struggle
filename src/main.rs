//! test_dx11
//!
//! some graphics tests
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod gi;
mod timer;

//#[doc(hidden)]
//#[global_allocator]
//static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[doc(hidden)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
	let event_loop: winit::event_loop::EventLoop<()> = winit::event_loop::EventLoop::new()?;
	let mut test: app::TestD3D11 = app::TestD3D11::default();

	event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
	event_loop.run_app(&mut test)?;

	return Ok(());
}
