//! Graphics Interface
//!
//! A module used to interact with GPU.

mod utils;

use windows::{
	core::Interface,
	Win32::{
		Foundation::{BOOL, HMODULE, HWND},
		Graphics::{Direct3D as d3d, Direct3D11 as d3d11, Dxgi as dxgi},
	},
};

#[allow(unused)]
enum AdapterKind {
	Unspecified,
	MinimumPower,
	HighPerformance,
}

impl AdapterKind {
	fn map_to_dxgi(&self) -> dxgi::DXGI_GPU_PREFERENCE {
		match self {
			| AdapterKind::Unspecified => dxgi::DXGI_GPU_PREFERENCE_UNSPECIFIED,
			| AdapterKind::MinimumPower => dxgi::DXGI_GPU_PREFERENCE_MINIMUM_POWER,
			| AdapterKind::HighPerformance => dxgi::DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE,
		}
	}
}

pub(crate) struct Adapter {
	device: d3d11::ID3D11Device5,
	context: d3d11::ID3D11DeviceContext4,
	rtv: Option<d3d11::ID3D11RenderTargetView>,

	factory: dxgi::IDXGIFactory7,
	swap_chain: Option<dxgi::IDXGISwapChain3>,
}

impl Adapter {
	pub(crate) unsafe fn new() -> Self {
		let factory: dxgi::IDXGIFactory7 =
			dxgi::CreateDXGIFactory().expect("failed to create dxgi factory");

		let adapter: dxgi::IDXGIAdapter4 =
			utils::select_adapter(&factory, AdapterKind::HighPerformance)
				.expect("failed to select adapter");

		let mut device: Option<d3d11::ID3D11Device> = None;
		let mut context: Option<d3d11::ID3D11DeviceContext> = None;

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
		.expect("failed to create device");

		let device = device
			.expect("failed to create a device")
			.cast()
			.expect("failed to cast a ID3D11Device to ID3D11Device5");
		let context = context
			.expect("failed to create a device context")
			.cast()
			.expect("failed to cast a ID3D11DeviceContext to ID3D11DeviceContext4");

		return Self {
			device,
			context,
			rtv: None,
			factory,
			swap_chain: None,
		};
	}

	pub(crate) unsafe fn create_swap_chain(
		&mut self, x: u32, y: u32, hwnd: &HWND,
	) -> Result<(), windows::core::Error> {
		let swap_chain: dxgi::IDXGISwapChain1 = self.factory.CreateSwapChainForHwnd(
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
		)?;

		self.rtv = Some(utils::back_buffer_rtv(&self.device, &swap_chain)?);
		self.swap_chain = Some(swap_chain.cast()?);

		return Ok(());
	}

	pub(crate) unsafe fn resize_buffers(
		&mut self, x: u32, y: u32,
	) -> Result<(), windows::core::Error> {
		self.context.Flush();
		self.rtv.take();

		if let Some(swap_chain) = &mut self.swap_chain {
			swap_chain.ResizeBuffers(
				2,
				x,
				y,
				dxgi::Common::DXGI_FORMAT_UNKNOWN,
				dxgi::DXGI_SWAP_CHAIN_FLAG(0),
			)?;

			self.rtv
				.replace(utils::back_buffer_rtv(&self.device, swap_chain)?);
		} else {
			return Err(windows::core::Error::new(
				windows::core::HRESULT(-1),
				"swap chain wasn't created",
			));
		}

		return Ok(());
	}

	pub(crate) unsafe fn present(&mut self) -> Result<(), windows::core::Error> {
		self.context.RSSetViewports(Some(&[d3d11::D3D11_VIEWPORT {
			TopLeftX: 0.0,
			TopLeftY: 0.0,
			Width: 800.0 as f32,
			Height: 600.0 as f32,
			MinDepth: 0.0,
			MaxDepth: 1.0,
		}]));

		if let Some(rtv) = &self.rtv {
			self.context
				.OMSetRenderTargets(Some(&[Some(rtv.clone())]), None);
			self.context
				.ClearRenderTargetView(rtv, &[0.0, 0.0, 0.0, 1.0]);
		} else {
			return Err(windows::core::Error::new(
				windows::core::HRESULT(-1),
				"rtv wasn't created",
			));
		}

		if let Some(swap_chain) = &self.swap_chain {
			swap_chain
				.Present1(
					1,
					dxgi::DXGI_PRESENT(0),
					&dxgi::DXGI_PRESENT_PARAMETERS::default(),
				)
				.ok()
				.expect("failed to present swapchain content");
		} else {
			return Err(windows::core::Error::new(
				windows::core::HRESULT(-1),
				"swap chain wasn't created",
			));
		}

		return Ok(());
	}
}
