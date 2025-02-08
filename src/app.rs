//! Application
//!
//! A module used to handle input and windowing

use winit::{
	application::ApplicationHandler,
	dpi::{LogicalPosition, LogicalSize},
	event::WindowEvent,
	event_loop::ActiveEventLoop,
	platform::windows::WindowAttributesExtWindows,
	raw_window_handle::{HasWindowHandle, RawWindowHandle},
	window::{Window, WindowAttributes, WindowId},
};

use crate::hal::cx::Context;

#[derive(Default)]
pub struct Struggle {
	context: Option<Context>,
	window: Option<Window>,
}

impl ApplicationHandler for Struggle {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let window: Window = event_loop
			.create_window(
				WindowAttributes::default()
					.with_active(true)
					.with_visible(false)
					.with_resizable(true)
					.with_window_icon(None)
					.with_taskbar_icon(None)
					.with_class_name("hello")
					.with_title("Hello, Direct3D 11!"),
			)
			.expect("failed to create a window");

		let Some(monitor) = window.current_monitor() else {
			return;
		};

		let scale_factor: LogicalSize<u32> =
			monitor.size().to_logical::<u32>(monitor.scale_factor());
		let size: (u32, u32) = (window.inner_size().width, window.inner_size().height);

		window.set_outer_position(LogicalPosition::new(
			(scale_factor.width - size.0) / 2,
			(scale_factor.height - size.1) / 2,
		));

		let RawWindowHandle::Win32(handle) = window
			.window_handle()
			.expect("failed to get window handle")
			.as_raw()
		else {
			panic!("expected a win32 window handle");
		};

		self.context = unsafe {
			Some(Context::new(windows::Win32::Foundation::HWND(
				handle.hwnd.get() as *mut std::ffi::c_void,
			)))
		};
		self.window = Some(window);
	}

	fn window_event(
		&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent,
	) {
		let Some(window) = &self.window else {
			return;
		};
		let Some(context) = &mut self.context else {
			return;
		};

		match event {
			| WindowEvent::CloseRequested => event_loop.exit(),
			| WindowEvent::RedrawRequested => {
				let viewport: LogicalSize<f32> =
					window.inner_size().to_logical::<f32>(window.scale_factor());
				let scissor: LogicalSize<i32> =
					window.inner_size().to_logical::<i32>(window.scale_factor());

				unsafe {
					context
						.present(viewport, scissor)
						.expect("failed to present frames");
				}

				window.request_redraw();
			}
			| WindowEvent::Resized(phys) => {
				let viewport: LogicalSize<f32> = phys.to_logical::<f32>(window.scale_factor());
				let scissor: LogicalSize<i32> = phys.to_logical::<i32>(window.scale_factor());

				if !window.is_visible().unwrap() {
					window.set_visible(true);
				} else {
					unsafe {
						context
							.resize_buffers(viewport, scissor)
							.expect("failed to resize swap chain buffers");
					}
				}

				window.request_redraw();
			}
			| _ => (),
		}
	}
}
