//! test_dx11
//!
//! A random graphics code written to learn Direct3D 11

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod gi;

#[doc(hidden)]
#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

use winit::{
	application::ApplicationHandler,
	event::WindowEvent,
	event_loop::ActiveEventLoop,
	platform::windows::WindowAttributesExtWindows,
	raw_window_handle::{HasWindowHandle, RawWindowHandle},
	window::{Window, WindowAttributes, WindowId},
};

struct TestD3D11 {
	adapter: crate::gi::Adapter,
	window: Option<Window>,
}

impl Default for TestD3D11 {
	fn default() -> Self {
		Self {
			adapter: unsafe { crate::gi::Adapter::new() },
			window: None,
		}
	}
}

impl ApplicationHandler for TestD3D11 {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let attributes: WindowAttributes = WindowAttributes::default()
			.with_active(true)
			.with_resizable(true)
			.with_window_icon(None)
			.with_taskbar_icon(None)
			.with_class_name("hello")
			.with_title("Hello, Direct3D 11!");

		let window: Window = event_loop
			.create_window(attributes)
			.expect("failed to create a window");

		if let Some(monitor) = window.current_monitor() {
			let sf: winit::dpi::LogicalSize<u32> =
				monitor.size().to_logical::<u32>(monitor.scale_factor());
			let size = (window.inner_size().width, window.inner_size().height);

			window.set_outer_position(winit::dpi::LogicalPosition::new(
				(sf.width - size.0) / 2,
				(sf.height - size.1) / 2,
			));
		}

		let RawWindowHandle::Win32(handle) = window.window_handle().unwrap().as_raw() else {
			unreachable!()
		};

		unsafe {
			self.adapter
				.create_swap_chain(
					window.inner_size().width,
					window.inner_size().height,
					&windows::Win32::Foundation::HWND(handle.hwnd.get() as *mut std::ffi::c_void),
				)
				.expect("failed to create swapchain");
		}

		self.window = Some(window);
	}

	fn window_event(
		&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent,
	) {
		match event {
			| WindowEvent::CloseRequested => event_loop.exit(),
			| WindowEvent::Resized(size) => {
				if let Some(window) = &self.window {
					unsafe {
						self.adapter
							.resize_buffers(size.width, size.height)
							.expect("failed to resize swap chain buffers")
					};

					window.request_redraw();
				}
			}
			| WindowEvent::RedrawRequested => {
				if let Some(window) = &self.window {
					unsafe { self.adapter.present().expect("failed to present frames") };

					window.request_redraw();
				}
			}
			| _ => (),
		}
	}
}

#[doc(hidden)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
	let event_loop: winit::event_loop::EventLoop<()> = winit::event_loop::EventLoop::new()?;
	let mut test: TestD3D11 = TestD3D11::default();

	event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
	event_loop.run_app(&mut test)?;

	return Ok(());
}
