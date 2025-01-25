//! Adapter
//!
//! A module used to select device, setup swap chain and present frames to the window.

use super::{BOOL, HMODULE, HWND, Interface, d3d, d3d11, dxgi, misc};
use crate::timer::Timer;

/// AdapterKind
///
/// Allows for specifying whether to prioritize power efficiency,
/// performance, or use the system's default preference.
#[allow(unused)]
pub(super) enum AdapterKind {
	/// The system will choose the adapter.
	Unspecified,
	/// Prefer the adapter with the lowest power consumption.
	MinimumPower,
	/// Prefer the adapter with the highest performance.
	HighPerformance,
}

impl AdapterKind {
	/// Maps the [`AdapterKind`] to the corresponding [`dxgi::DXGI_GPU_PREFERENCE`] value.
	pub(super) fn map_to_dxgi(&self) -> dxgi::DXGI_GPU_PREFERENCE {
		match self {
			| Self::Unspecified => dxgi::DXGI_GPU_PREFERENCE_UNSPECIFIED,
			| Self::MinimumPower => dxgi::DXGI_GPU_PREFERENCE_MINIMUM_POWER,
			| Self::HighPerformance => dxgi::DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE,
		}
	}
}

/// Adapter
///
/// A wrapper for device, context and resources.
pub(crate) struct Context {
	/// D3D11 device
	device: d3d11::ID3D11Device5,
	/// D3D11 device context
	cmd_list: d3d11::ID3D11DeviceContext4,
	/// RenderTargetView derived from back buffer
	back_buffer_rtv: Option<d3d11::ID3D11RenderTargetView>,

	/// Time value for animations etc
	time: f32,
	/// Timer tracks the time elapsed between frames
	timer: Timer,

	/// DXGI factory
	factory: dxgi::IDXGIFactory7,
	/// DXGI swap chain
	swap_chain: Option<dxgi::IDXGISwapChain3>,
}

impl core::default::Default for Context {
	fn default() -> Self {
		return Self::new();
	}
}

impl Context {
	/// Constructor
	pub(crate) fn new() -> Self {
		let factory: dxgi::IDXGIFactory7 = unsafe {
			dxgi::CreateDXGIFactory2(dxgi::DXGI_CREATE_FACTORY_DEBUG)
				.expect("failed to create dxgi factory")
		};

		let adapter: dxgi::IDXGIAdapter4 =
			misc::select_adapter(&factory, AdapterKind::HighPerformance)
				.expect("failed to select adapter");

		let mut device: Option<d3d11::ID3D11Device> = None;
		let mut context: Option<d3d11::ID3D11DeviceContext> = None;

		unsafe {
			d3d11::D3D11CreateDevice(
				&adapter,
				d3d::D3D_DRIVER_TYPE_UNKNOWN,
				HMODULE(std::ptr::null_mut()),
				if cfg!(debug_assertions) {
					d3d11::D3D11_CREATE_DEVICE_DEBUG
				} else {
					d3d11::D3D11_CREATE_DEVICE_FLAG(0)
				},
				Some(&[d3d::D3D_FEATURE_LEVEL_11_0]),
				d3d11::D3D11_SDK_VERSION,
				Some(&mut device),
				None,
				Some(&mut context),
			)
			.expect("failed to create device")
		};

		let device: d3d11::ID3D11Device5 = device
			.expect("failed to create a device")
			.cast()
			.expect("failed to cast a ID3D11Device to ID3D11Device5");
		let context: d3d11::ID3D11DeviceContext4 = context
			.expect("failed to create a device context")
			.cast()
			.expect("failed to cast a ID3D11DeviceContext to ID3D11DeviceContext4");

		Self {
			factory,
			device,
			cmd_list: context,
			back_buffer_rtv: None,
			time: 0.0,
			timer: Timer::new(),
			swap_chain: None,
		}
	}

	pub(crate) fn set_swap_chain(
		&mut self, x: u32, y: u32, hwnd: &HWND,
	) -> Result<(), windows::core::Error> {
		let swap_chain: dxgi::IDXGISwapChain1 = unsafe {
			self.factory.CreateSwapChainForHwnd(
				&self.device,
				*hwnd,
				&dxgi::DXGI_SWAP_CHAIN_DESC1 {
					Width: x,
					Height: y,
					Format: dxgi::Common::DXGI_FORMAT_R8G8B8A8_UNORM,
					SampleDesc: dxgi::Common::DXGI_SAMPLE_DESC {
						Count: 1,
						Quality: 0,
					},
					BufferUsage: dxgi::DXGI_USAGE_RENDER_TARGET_OUTPUT,
					BufferCount: 3,
					SwapEffect: dxgi::DXGI_SWAP_EFFECT_FLIP_DISCARD,
					Flags: 0,
					..dxgi::DXGI_SWAP_CHAIN_DESC1::default()
				},
				Some(&dxgi::DXGI_SWAP_CHAIN_FULLSCREEN_DESC {
					Windowed: BOOL::from(true),
					..dxgi::DXGI_SWAP_CHAIN_FULLSCREEN_DESC::default()
				}),
				None,
			)
		}?;

		unsafe {
			self.factory
				.MakeWindowAssociation(*hwnd, dxgi::DXGI_MWA_NO_ALT_ENTER)?;
		}

		self.back_buffer_rtv = Some(misc::back_buffer_rtv(&self.device, &swap_chain)?);
		self.swap_chain = Some(swap_chain.cast()?);

		return Ok(());
	}

	pub(crate) fn resize_buffers(&mut self, x: u32, y: u32) -> Result<(), windows::core::Error> {
		self.back_buffer_rtv.take();
		unsafe { self.cmd_list.Flush() };

		match &mut self.swap_chain {
			| Some(swap_chain) => unsafe {
				swap_chain.ResizeBuffers(
					0,
					x,
					y,
					dxgi::Common::DXGI_FORMAT_UNKNOWN,
					dxgi::DXGI_SWAP_CHAIN_FLAG(0),
				)?;

				self.back_buffer_rtv
					.replace(misc::back_buffer_rtv(&self.device, swap_chain)?);
				self.cmd_list.RSSetViewports(Some(&[d3d11::D3D11_VIEWPORT {
					TopLeftX: 0.0,
					TopLeftY: 0.0,
					Width: x as f32,
					Height: y as f32,
					MinDepth: 0.0,
					MaxDepth: 1.0,
				}]))
			},
			| None => {
				return Err(windows::core::Error::new(
					windows::core::HRESULT(-1),
					"swap chain wasn't created",
				));
			}
		}

		return Ok(());
	}

	pub(crate) fn present(&mut self) -> Result<(), windows::core::Error> {
		self.timer.update();
		self.time += self.timer.delta.as_secs_f32();

		let (r, g, b, a) =
			crate::gi::misc::hsla_to_rgba(self.time * std::f32::consts::PI * 40.0, 0.4, 0.8, 1.0);

		match &self.swap_chain {
			| Some(swap_chain) => unsafe {
				self.cmd_list.RSSetViewports(Some(&[d3d11::D3D11_VIEWPORT {
					TopLeftX: 0.0,
					TopLeftY: 0.0,
					Width: swap_chain.GetDesc1()?.Width as f32,
					Height: swap_chain.GetDesc1()?.Height as f32,
					MinDepth: 0.0,
					MaxDepth: 1.0,
				}]));
			},
			| None => {
				return Err(windows::core::Error::new(
					windows::core::HRESULT(-1),
					"swap chain wasn't created",
				));
			}
		}

		match &self.back_buffer_rtv {
			| Some(rtv) => unsafe {
				self.cmd_list.ClearRenderTargetView(rtv, &[r, g, b, a]);
				self.cmd_list
					.OMSetRenderTargets(Some(&[Some(rtv.clone())]), None);
			},
			| None => {
				return Err(windows::core::Error::new(
					windows::core::HRESULT(-1),
					"rtv wasn't created",
				));
			}
		}

		match &self.swap_chain {
			| Some(swap_chain) => unsafe {
				swap_chain
					.Present1(
						1,
						dxgi::DXGI_PRESENT(0),
						&dxgi::DXGI_PRESENT_PARAMETERS::default(),
					)
					.ok()
					.expect("failed to present swapchain content");
			},
			| None => {
				return Err(windows::core::Error::new(
					windows::core::HRESULT(-1),
					"swap chain wasn't created",
				));
			}
		}

		return Ok(());
	}
}
