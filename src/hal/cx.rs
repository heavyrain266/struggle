//! `Context`
//!
//! A module used to select device, setup swap chain and present frames to the window.

use windows::core::HSTRING;

use super::{BOOL, HMODULE, HWND, Interface, d3d, d3d11, dxgi, misc, shared};
use crate::timer::Timer;

#[allow(unused)]
pub enum AdapterKind {
	Unspecified,
	MinimumPower,
	HighPerformance,
}

impl AdapterKind {
	pub const fn map_to_dxgi(&self) -> dxgi::DXGI_GPU_PREFERENCE {
		match self {
			| Self::Unspecified => dxgi::DXGI_GPU_PREFERENCE_UNSPECIFIED,
			| Self::MinimumPower => dxgi::DXGI_GPU_PREFERENCE_MINIMUM_POWER,
			| Self::HighPerformance => dxgi::DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE,
		}
	}
}

pub struct Context {
	device: d3d11::ID3D11Device5,
	cmd_list: d3d11::ID3D11DeviceContext4,
	backbuffer_rt_view: Option<d3d11::ID3D11RenderTargetView>,
	factory: dxgi::IDXGIFactory7,
	swap_chain: Option<dxgi::IDXGISwapChain3>,

	input_layout: d3d11::ID3D11InputLayout,
	pixel_shader: d3d11::ID3D11PixelShader,
	vertex_shader: d3d11::ID3D11VertexShader,

	time: f32,
	timer: Timer,
}

impl core::default::Default for Context {
	fn default() -> Self {
		unsafe { Self::new() }
	}
}

impl Context {
	pub unsafe fn new() -> Self {
		let factory: dxgi::IDXGIFactory7 = unsafe {
			dxgi::CreateDXGIFactory2(dxgi::DXGI_CREATE_FACTORY_DEBUG)
				.expect("failed to create dxgi factory")
		};

		let adapter: dxgi::IDXGIAdapter4 = unsafe {
			misc::select_adapter(&factory, &AdapterKind::HighPerformance)
				.expect("failed to select adapter")
		};

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
			.expect("failed to create device");
		};

		let device: d3d11::ID3D11Device5 = device
			.expect("failed to create a device")
			.cast()
			.expect("failed to cast a ID3D11Device to ID3D11Device5");
		let context: d3d11::ID3D11DeviceContext4 = context
			.expect("failed to create a device context")
			.cast()
			.expect("failed to cast a ID3D11DeviceContext to ID3D11DeviceContext4");

		let shaders: HSTRING = std::path::Path::new(&format!(
			"{}/redist/shaders/shaders.hlsl",
			env!("CARGO_MANIFEST_DIR")
		))
		.to_str()
		.unwrap()
		.into();

		let vs_blob: d3d::ID3DBlob = unsafe {
			misc::compile(&shaders, &misc::ShaderKind::Vertex)
				.expect("failed to compile vertex shader")
		};
		let ps_blob: d3d::ID3DBlob = unsafe {
			misc::compile(&shaders, &misc::ShaderKind::Pixel)
				.expect("failed to compile pixel shader")
		};

		let mut vs: Option<d3d11::ID3D11VertexShader> = None;
		let mut ps: Option<d3d11::ID3D11PixelShader> = None;

		let mut input_layout: Option<d3d11::ID3D11InputLayout> = None;

		unsafe {
			device
				.CreateVertexShader(
					std::slice::from_raw_parts(
						vs_blob.GetBufferPointer() as *const u8,
						vs_blob.GetBufferSize(),
					),
					None,
					Some(&mut vs),
				)
				.expect("failed to create vertex shader");
			device
				.CreatePixelShader(
					std::slice::from_raw_parts(
						ps_blob.GetBufferPointer() as *const u8,
						ps_blob.GetBufferSize(),
					),
					None,
					Some(&mut ps),
				)
				.expect("failed to create pixel shader");

			device
				.CreateInputLayout(
					&[
						d3d11::D3D11_INPUT_ELEMENT_DESC {
							SemanticName: windows::core::s!("POSITION"),
							SemanticIndex: 0,
							Format: dxgi::Common::DXGI_FORMAT_R32G32_FLOAT,
							InputSlot: 0,
							AlignedByteOffset: std::mem::offset_of!(shared::Vertex, position)
								as u32,
							InputSlotClass: d3d11::D3D11_INPUT_PER_VERTEX_DATA,
							InstanceDataStepRate: 0,
						},
						d3d11::D3D11_INPUT_ELEMENT_DESC {
							SemanticName: windows::core::s!("COLOR"),
							SemanticIndex: 0,
							Format: dxgi::Common::DXGI_FORMAT_R32G32B32A32_FLOAT,
							InputSlot: 0,
							AlignedByteOffset: std::mem::offset_of!(shared::Vertex, color) as u32,
							InputSlotClass: d3d11::D3D11_INPUT_PER_VERTEX_DATA,
							InstanceDataStepRate: 0,
						},
					],
					std::slice::from_raw_parts(
						vs_blob.GetBufferPointer() as *const u8,
						vs_blob.GetBufferSize(),
					),
					Some(&mut input_layout),
				)
				.expect("failed to create input layout");
		}

		Self {
			factory,
			device,
			cmd_list: context,
			backbuffer_rt_view: None,
			input_layout: input_layout.unwrap(),
			vertex_shader: vs.unwrap(),
			pixel_shader: ps.unwrap(),
			time: 0.0,
			timer: Timer::new(),
			swap_chain: None,
		}
	}

	pub fn set_swap_chain(&mut self, hwnd: HWND) -> Result<(), windows::core::Error> {
		let swap_chain: dxgi::IDXGISwapChain1 = unsafe {
			self.factory.CreateSwapChainForHwnd(
				&self.device,
				hwnd,
				&dxgi::DXGI_SWAP_CHAIN_DESC1 {
					Width: 0,
					Height: 0,
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
				.MakeWindowAssociation(hwnd, dxgi::DXGI_MWA_NO_ALT_ENTER)?;
			self.backbuffer_rt_view = Some(misc::framebuffer_rtv(&self.device, &swap_chain)?);
		}

		self.swap_chain = Some(swap_chain.cast()?);

		return Ok(());
	}

	pub unsafe fn resize_buffers(&mut self, x: f32, y: f32) -> Result<(), windows::core::Error> {
		let Some(swap_chain) = &mut self.swap_chain else {
			return Err(windows::core::Error::new(
				windows::core::HRESULT(-1),
				"swap chain wasn't created",
			));
		};

		self.backbuffer_rt_view.take();

		unsafe {
			self.cmd_list.Flush();

			swap_chain.ResizeBuffers(
				0,
				0,
				0,
				dxgi::Common::DXGI_FORMAT_UNKNOWN,
				dxgi::DXGI_SWAP_CHAIN_FLAG(0),
			)?;

			self.cmd_list.RSSetViewports(Some(&[d3d11::D3D11_VIEWPORT {
				TopLeftX: 0.0,
				TopLeftY: 0.0,
				Width: x,
				Height: y,
				MinDepth: 0.0,
				MaxDepth: 1.0,
			}]));
			self.backbuffer_rt_view
				.replace(misc::framebuffer_rtv(&self.device, swap_chain)?);
		}

		return Ok(());
	}

	pub unsafe fn present(&mut self, x: f32, y: f32) -> Result<(), windows::core::Error> {
		self.timer.update();
		self.time += self.timer.delta.as_secs_f32();

		let rgba: [f32; 4] =
			misc::hsla_to_rgba(&[self.time * std::f32::consts::PI * 40.0, 0.4, 0.7, 1.0]);
		let Some(swap_chain) = &self.swap_chain else {
			return Err(windows::core::Error::new(
				windows::core::HRESULT(-1),
				"swap chain wasn't created",
			));
		};
		let Some(rtv) = &self.backbuffer_rt_view else {
			return Err(windows::core::Error::new(
				windows::core::HRESULT(-1),
				"back buffer RTV wasn't created",
			));
		};

		let vertices: [shared::Vertex; 3] = [
			shared::Vertex::new([0.0, 0.5, 0.0], [1.0, 0.0, 0.0, 1.0]),
			shared::Vertex::new([0.5, -0.5, 0.0], [0.0, 1.0, 0.0, 1.0]),
			shared::Vertex::new([-0.5, -0.5, 0.0], [0.0, 0.0, 1.0, 1.0]),
		];
		let vertex_buffer: d3d11::ID3D11Buffer = shared::buffer(&vertices, 1, &self.device)?;
		let index_buffer: d3d11::ID3D11Buffer = shared::buffer(&[0, 1, 2], 2, &self.device)?;

		unsafe {
			self.cmd_list
				.IASetPrimitiveTopology(d3d::D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
			self.cmd_list.IASetInputLayout(&self.input_layout);
			self.cmd_list.IASetVertexBuffers(
				0,
				1,
				Some(&Some(vertex_buffer)),
				Some(&(std::mem::size_of::<shared::Vertex>() as u32)),
				Some(&0),
			);
			self.cmd_list
				.IASetIndexBuffer(&index_buffer, dxgi::Common::DXGI_FORMAT_R32_UINT, 0);
			self.cmd_list.VSSetShader(&self.vertex_shader, None);
			self.cmd_list.PSSetShader(&self.pixel_shader, None);
			self.cmd_list.RSSetViewports(Some(&[d3d11::D3D11_VIEWPORT {
				TopLeftX: 0.0,
				TopLeftY: 0.0,
				Width: x,
				Height: y,
				MinDepth: 0.0,
				MaxDepth: 1.0,
			}]));
			self.cmd_list
				.OMSetRenderTargets(Some(&[Some(rtv.clone())]), None);
			self.cmd_list.ClearRenderTargetView(rtv, &rgba);
			self.cmd_list.DrawIndexed(3, 0, 0);

			swap_chain
				.Present1(
					1,
					dxgi::DXGI_PRESENT(0),
					&dxgi::DXGI_PRESENT_PARAMETERS::default(),
				)
				.ok()
				.expect("failed to present swapchain content");
		};

		return Ok(());
	}
}
