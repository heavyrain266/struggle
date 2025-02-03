//! Misc
//!
//! A tiny module with miscellaneous utils.

#![allow(unused)]

use super::{HSTRING, d3d, d3d11, dxgi, fxc, s};

pub enum ShaderKind {
	Pixel,
	Vertex,
}

pub unsafe fn compile(
	file: &HSTRING, kind: &ShaderKind,
) -> Result<d3d::ID3DBlob, windows::core::Error> {
	let flags: u32 = if cfg!(debug_assertions) {
		fxc::D3DCOMPILE_DEBUG | fxc::D3DCOMPILE_SKIP_OPTIMIZATION
	} else {
		0
	};

	let mut pixel: Option<d3d::ID3DBlob> = None;
	let mut vertex: Option<d3d::ID3DBlob> = None;

	match kind {
		| ShaderKind::Pixel => {
			let pixel: d3d::ID3DBlob = unsafe {
				fxc::D3DCompileFromFile(
					file,
					None,
					None,
					s!("ps_main"),
					s!("ps_5_0"),
					flags,
					0,
					&mut pixel,
					None,
				)
				.map(|()| pixel.expect("failed to map pixel shader"))?
			};

			return Ok(pixel);
		}
		| ShaderKind::Vertex => {
			let vertex: d3d::ID3DBlob = unsafe {
				fxc::D3DCompileFromFile(
					file,
					None,
					None,
					s!("vs_main"),
					s!("vs_5_0"),
					flags,
					0,
					&mut vertex,
					None,
				)
				.map(|()| vertex.expect("failed to map vertex shader"))?
			};

			return Ok(vertex);
		}
	}
}

#[inline(always)]
pub unsafe fn framebuffer_rtv(
	device: &d3d11::ID3D11Device, swap_chain: &dxgi::IDXGISwapChain1,
) -> Result<d3d11::ID3D11RenderTargetView, windows::core::Error> {
	let mut rt_view: Option<d3d11::ID3D11RenderTargetView> = None;

	unsafe {
		device.CreateRenderTargetView(
			&swap_chain.GetBuffer::<d3d11::ID3D11Texture2D>(0)?,
			Some(
				&(d3d11::D3D11_RENDER_TARGET_VIEW_DESC {
					Format: dxgi::Common::DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
					ViewDimension: d3d11::D3D11_RTV_DIMENSION_TEXTURE2D,
					..Default::default()
				}),
			),
			Some(&mut rt_view),
		)?
	};

	return Ok(rt_view.unwrap());
}

#[inline(always)]
pub unsafe fn select_adapter(
	factory: &dxgi::IDXGIFactory7, prefer: &super::context::AdapterKind,
) -> Result<dxgi::IDXGIAdapter4, windows::core::Error> {
	for i in 0.. {
		let adapter: dxgi::IDXGIAdapter4 =
			unsafe { factory.EnumAdapterByGpuPreference(i, prefer.map_to_dxgi()) }?;

		if unsafe { adapter.GetDesc3() }?.Flags & dxgi::DXGI_ADAPTER_FLAG3_SOFTWARE
			!= dxgi::DXGI_ADAPTER_FLAG3_NONE
		{
			continue;
		}

		return Ok(adapter);
	}

	// fallback to software rasterizer (WARP)
	unsafe { factory.EnumWarpAdapter() }
}

#[inline(always)]
pub fn hsla_to_rgba(hsla: &[f32; 4]) -> [f32; 4] {
	let (hue, saturation, lightness, alpha): (f32, f32, f32, f32) = (
		hsla[0].rem_euclid(360.0),
		hsla[1].clamp(0.0, 1.0),
		hsla[2].clamp(0.0, 1.0),
		hsla[3].clamp(0.0, 1.0),
	);

	let chroma: f32 = ((1.0 - 2.0f32.mul_add(lightness, -1.0)) * saturation).clamp(0.0, 1.0);
	let imval: f32 = chroma * (1.0 - ((hue / 60.0).rem_euclid(2.0) - 1.0).abs());
	let offset: f32 = lightness - chroma / 2.0;

	let (red, green, blue): (f32, f32, f32) = match (hue as u32) / 60 {
		| 0 => (chroma, imval, 0.0),
		| 1 => (imval, chroma, 0.0),
		| 2 => (0.0, chroma, imval),
		| 3 => (0.0, imval, chroma),
		| 4 => (imval, 0.0, chroma),
		| _ => (chroma, 0.0, imval),
	};

	[red + offset, green + offset, blue + offset, alpha]
}
