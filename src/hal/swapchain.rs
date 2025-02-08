use super::{BOOL, HWND, Interface, adapter::Adapter, dxgi};

pub struct SwapChain {
	pub handle: dxgi::IDXGISwapChain3,
}

impl SwapChain {
	pub unsafe fn new(
		factory: &dxgi::IDXGIFactory7, adapter: &Adapter, hwnd: HWND,
	) -> Result<Self, windows::core::Error> {
		Ok(Self {
			handle: unsafe {
				factory.CreateSwapChainForHwnd(
					&adapter.device,
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
			}?
			.cast()?,
		})
	}
}
