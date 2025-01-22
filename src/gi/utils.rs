//! Utils
//!
//! A tiny module with miscellaneous utils.

use windows::Win32::Graphics::{Direct3D11 as d3d11, Dxgi as dxgi};

pub(super) unsafe fn back_buffer_rtv(
	device: &d3d11::ID3D11Device, swap_chain: &dxgi::IDXGISwapChain1,
) -> windows::core::Result<d3d11::ID3D11RenderTargetView> {
	let back_buffer: d3d11::ID3D11Texture2D = swap_chain.GetBuffer::<d3d11::ID3D11Texture2D>(0)?;
	let mut rtv: Option<d3d11::ID3D11RenderTargetView> = None;

	device.CreateRenderTargetView(&back_buffer, None, Some(&mut rtv))?;

	Ok(rtv.unwrap())
}

pub(super) unsafe fn select_adapter(
	factory: &dxgi::IDXGIFactory7, kind: super::AdapterKind,
) -> Result<dxgi::IDXGIAdapter4, windows::core::Error> {
	for i in 0.. {
		let adapter: dxgi::IDXGIAdapter4 =
			factory.EnumAdapterByGpuPreference(i, kind.map_to_dxgi())?;

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
