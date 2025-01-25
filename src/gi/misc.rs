//! Misc
//!
//! A tiny module with miscellaneous utils.

#![allow(unused)]

use super::{d3d, d3d11, dxgi, fxc, s, HSTRING};

pub(super) enum ShaderKind {
	Pixel = 0,
	Vertex = 1,
}

pub(super) unsafe fn compile(
	file: &HSTRING, kind: ShaderKind,
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
			let pixel: d3d::ID3DBlob = fxc::D3DCompileFromFile(
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
			.map(|()| pixel.expect("failed to map pixel shader"))?;

			Ok(pixel)
		}
		| ShaderKind::Vertex => {
			let vertex: d3d::ID3DBlob = fxc::D3DCompileFromFile(
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
			.map(|()| vertex.expect("failed to map vertex shader"))?;

			Ok(vertex)
		}
	}
}

pub(super) unsafe fn back_buffer_rtv(
	device: &d3d11::ID3D11Device, swap_chain: &dxgi::IDXGISwapChain1,
) -> Result<d3d11::ID3D11RenderTargetView, windows::core::Error> {
	let back_buffer: d3d11::ID3D11Texture2D = swap_chain.GetBuffer::<d3d11::ID3D11Texture2D>(0)?;
	let mut rtv: Option<d3d11::ID3D11RenderTargetView> = None;

	device.CreateRenderTargetView(&back_buffer, None, Some(&mut rtv))?;

	Ok(rtv.unwrap())
}

pub(super) unsafe fn select_adapter(
	factory: &dxgi::IDXGIFactory7, prefer: super::adapter::AdapterKind,
) -> Result<dxgi::IDXGIAdapter4, windows::core::Error> {
	for i in 0.. {
		let adapter: dxgi::IDXGIAdapter4 =
			factory.EnumAdapterByGpuPreference(i, prefer.map_to_dxgi())?;

		if adapter.GetDesc3()?.Flags & dxgi::DXGI_ADAPTER_FLAG3_SOFTWARE
			!= dxgi::DXGI_ADAPTER_FLAG3_NONE
		{
			continue;
		}

		return Ok(adapter);
	}

	// fallback to software rasterizer (WARP)
	factory.EnumWarpAdapter()
}

pub(crate) fn hsla_to_rgba(
	hue: f32, saturation: f32, lightness: f32, alpha: f32,
) -> (f32, f32, f32, f32) {
	let (h, s, l): (f32, f32, f32) = (
		hue.rem_euclid(360.0),
		saturation.clamp(0.0, 1.0),
		lightness.clamp(0.0, 1.0),
	);

	let c: f32 = (1.0 - (2.0 * l - 1.0).abs()) * s;
	let x: f32 = c * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
	let m: f32 = l - c / 2.0;

	let (r, g, b) = match (h as u32) / 60 {
		| 0 => (c, x, 0.0),
		| 1 => (x, c, 0.0),
		| 2 => (0.0, c, x),
		| 3 => (0.0, x, c),
		| 4 => (x, 0.0, c),
		| _ => (c, 0.0, x),
	};

	return (r + m, g + m, b + m, alpha);
}
