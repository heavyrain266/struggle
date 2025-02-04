//! `Context`
//!
//! A module used to select device, setup swap chain and present frames to the window.

use super::{
	BOOL, HSTRING, HWND, RECT, adapter::Adapter, d3d, d3d11, dxgi, misc, shared,
	swaochain::SwapChain,
};
use crate::timer::Timer;

pub struct Context {
	adapter: Adapter,
	swap_chain: SwapChain,
	backbuffer_rt_view: Option<d3d11::ID3D11RenderTargetView>,

	input_layout: d3d11::ID3D11InputLayout,
	pixel_shader: d3d11::ID3D11PixelShader,
	vertex_shader: d3d11::ID3D11VertexShader,
	raster_state: d3d11::ID3D11RasterizerState2,

	time: f32,
	timer: Timer,
}

impl Context {
	pub unsafe fn new(hwnd: HWND) -> Self {
		let factory: dxgi::IDXGIFactory7 = unsafe {
			dxgi::CreateDXGIFactory2(dxgi::DXGI_CREATE_FACTORY_DEBUG)
				.expect("failed to create dxgi factory")
		};

		let adapter: Adapter = unsafe { Adapter::new(&factory) };
		let swap_chain = unsafe { SwapChain::new(&factory, &adapter, hwnd) }
			.expect("failed to create a swap chain");
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
		let mut raster_state: Option<d3d11::ID3D11RasterizerState2> = None;

		unsafe {
			adapter
				.device
				.CreateVertexShader(
					std::slice::from_raw_parts(
						vs_blob.GetBufferPointer() as *const u8,
						vs_blob.GetBufferSize(),
					),
					None,
					Some(&mut vs),
				)
				.expect("failed to create vertex shader");
			adapter
				.device
				.CreatePixelShader(
					std::slice::from_raw_parts(
						ps_blob.GetBufferPointer() as *const u8,
						ps_blob.GetBufferSize(),
					),
					None,
					Some(&mut ps),
				)
				.expect("failed to create pixel shader");

			adapter
				.device
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

			adapter
				.device
				.CreateRasterizerState2(
					&d3d11::D3D11_RASTERIZER_DESC2 {
						FillMode: d3d11::D3D11_FILL_SOLID,
						CullMode: d3d11::D3D11_CULL_NONE,
						FrontCounterClockwise: BOOL::from(false),
						DepthBias: 0,
						DepthBiasClamp: 0.0,
						SlopeScaledDepthBias: 0.0,
						DepthClipEnable: BOOL::from(false),
						ScissorEnable: BOOL::from(true),
						MultisampleEnable: BOOL::from(false),
						AntialiasedLineEnable: BOOL::from(false),
						ForcedSampleCount: 1,
						ConservativeRaster: d3d11::D3D11_CONSERVATIVE_RASTERIZATION_MODE_OFF,
					},
					Some(&mut raster_state),
				)
				.expect("failed to create rasterizer state");
		}

		Self {
			adapter,
			swap_chain,
			backbuffer_rt_view: None,
			input_layout: input_layout.unwrap(),
			vertex_shader: vs.unwrap(),
			pixel_shader: ps.unwrap(),
			raster_state: raster_state.unwrap(),
			time: 0.0,
			timer: Timer::new(),
		}
	}

	pub unsafe fn resize_buffers(
		&mut self, x: f32, y: f32, scissor: winit::dpi::LogicalSize<i32>,
	) -> Result<(), windows::core::Error> {
		self.backbuffer_rt_view.take();

		unsafe {
			self.adapter.context.Flush();
			self.swap_chain.handle.ResizeBuffers(
				0,
				0,
				0,
				dxgi::Common::DXGI_FORMAT_UNKNOWN,
				dxgi::DXGI_SWAP_CHAIN_FLAG(0),
			)?;

			self.adapter
				.context
				.RSSetViewports(Some(&[d3d11::D3D11_VIEWPORT {
					TopLeftX: 0.0,
					TopLeftY: 0.0,
					Width: x,
					Height: y,
					MinDepth: 0.0,
					MaxDepth: 1.0,
				}]));
			self.adapter.context.RSSetScissorRects(Some(&[RECT {
				left: 0,
				top: 0,
				right: scissor.width,
				bottom: scissor.height,
			}]));

			self.backbuffer_rt_view.replace(misc::backbuffer_rt_view(
				&self.adapter.device,
				&self.swap_chain.handle,
			)?);
		}

		return Ok(());
	}

	pub unsafe fn present(
		&mut self, x: f32, y: f32, scissor: winit::dpi::LogicalSize<i32>,
	) -> Result<(), windows::core::Error> {
		self.timer.update();
		self.time += self.timer.delta.as_secs_f32();

		let rgba: [f32; 4] =
			misc::hsla_to_rgba(&[self.time * std::f32::consts::PI * 40.0, 0.4, 0.7, 1.0]);
		let Some(rtv) = &self.backbuffer_rt_view else {
			return Err(windows::core::Error::new(
				windows::core::HRESULT(-1),
				"back buffer RTV wasn't created",
			));
		};

		let vertices: [shared::Vertex; 3] = [
			shared::Vertex::new([0.0, 0.5, 0.0], rgba),
			shared::Vertex::new([0.5, -0.5, 0.0], rgba),
			shared::Vertex::new([-0.5, -0.5, 0.0], rgba),
		];
		let vertex_buffer: d3d11::ID3D11Buffer =
			unsafe { shared::buffer(&vertices, 1, &self.adapter.device) }?;
		let index_buffer: d3d11::ID3D11Buffer =
			unsafe { shared::buffer(&[0, 1, 2, 0], 2, &self.adapter.device) }?;

		unsafe {
			self.adapter
				.context
				.IASetPrimitiveTopology(d3d::D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
			self.adapter.context.IASetInputLayout(&self.input_layout);
			self.adapter.context.IASetVertexBuffers(
				0,
				1,
				Some(&Some(vertex_buffer)),
				Some(&(std::mem::size_of::<shared::Vertex>() as u32)),
				Some(&0),
			);
			self.adapter.context.IASetIndexBuffer(
				&index_buffer,
				dxgi::Common::DXGI_FORMAT_R32_UINT,
				0,
			);
			self.adapter.context.VSSetShader(&self.vertex_shader, None);
			self.adapter.context.RSSetState(&self.raster_state);
			self.adapter
				.context
				.RSSetViewports(Some(&[d3d11::D3D11_VIEWPORT {
					TopLeftX: 0.0,
					TopLeftY: 0.0,
					Width: x,
					Height: y,
					MinDepth: 0.0,
					MaxDepth: 1.0,
				}]));
			self.adapter.context.RSSetScissorRects(Some(&[RECT {
				left: 0,
				top: 0,
				right: scissor.width,
				bottom: scissor.height,
			}]));
			self.adapter.context.PSSetShader(&self.pixel_shader, None);
			self.adapter
				.context
				.OMSetRenderTargets(Some(&[Some(rtv.clone())]), None);
			self.adapter
				.context
				.ClearRenderTargetView(rtv, &[0.0, 0.0, 0.0, 1.0]);
			self.adapter.context.DrawIndexed(4, 0, 0);

			self.swap_chain
				.handle
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
