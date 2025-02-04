use super::{HMODULE, Interface, d3d, d3d11, dxgi, misc};

pub struct Adapter {
	pub device: d3d11::ID3D11Device5,
	pub context: d3d11::ID3D11DeviceContext4,
}

#[allow(unused)]
pub enum AdapterKind {
	Unspecified,
	MinimumPower,
	HighPerformance,
}

impl AdapterKind {
	pub const fn map_to_dxgi(&self) -> dxgi::DXGI_GPU_PREFERENCE {
		return match self {
			| Self::Unspecified => dxgi::DXGI_GPU_PREFERENCE_UNSPECIFIED,
			| Self::MinimumPower => dxgi::DXGI_GPU_PREFERENCE_MINIMUM_POWER,
			| Self::HighPerformance => dxgi::DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE,
		};
	}
}

impl Adapter {
	pub unsafe fn new(factory: &dxgi::IDXGIFactory7) -> Self {
		let mut device: Option<d3d11::ID3D11Device> = None;
		let mut context: Option<d3d11::ID3D11DeviceContext> = None;

		let adapter: dxgi::IDXGIAdapter4 = unsafe {
			misc::select_adapter(factory, &AdapterKind::HighPerformance)
				.expect("failed to select adapter")
		};

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

		return Self {
			device: device
				.expect("failed to create a device")
				.cast()
				.expect("failed to cast a ID3D11Device to ID3D11Device5"),
			context: context
				.expect("failed to create a device context")
				.cast()
				.expect("failed to cast a ID3D11DeviceContext to ID3D11DeviceContext4"),
		};
	}
}
