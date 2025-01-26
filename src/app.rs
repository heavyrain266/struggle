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

use crate::hal::context::Context;

/// DirectX 11 Test
///
/// Encapsulates the components for testing D3D11 rendering
#[derive(Default)]
pub struct TestD3D11 {
	/// A graphics adapter used for rendering.
	context: Context,
	/// An optional window for rendering.
	window: Option<Window>,
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

		self.context
			.set_swap_chain(windows::Win32::Foundation::HWND(
				handle.hwnd.get() as *mut std::ffi::c_void
			))
			.expect("failed to create swapchain");

		self.window = Some(window);
	}

	fn window_event(
		&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent,
	) {
		let Some(window) = &self.window else {
			return;
		};

		match event {
			| WindowEvent::CloseRequested => event_loop.exit(),
			| WindowEvent::Resized(size) => {
				let lsize: LogicalSize<f32> = size.to_logical::<f32>(window.scale_factor());

				if lsize.width != 0.0 && lsize.height != 0.0 {
					self.context
						.resize_buffers(lsize.width, lsize.height)
						.expect("failed to resize swap chain buffers");
				}

				window.request_redraw();
			}
			| WindowEvent::RedrawRequested => {
				self.context.present().expect("failed to present frames");

				window.request_redraw();
			}
			| _ => (),
		}
	}
}
