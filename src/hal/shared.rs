//! Shared
//!
//! A module used to share data between CPU and GPU

use super::d3d11;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
	pub position: [f32; 3],
	pub color: [f32; 4],
}

impl Vertex {
	#[inline]
	pub const fn new(position: [f32; 3], color: [f32; 4]) -> Self {
		return Self { position, color };
	}
}

pub fn buffer<T>(
	data: &[T], bind_flags: u32, device: &d3d11::ID3D11Device,
) -> Result<d3d11::ID3D11Buffer, windows::core::Error> {
	let size: u32 = u32::try_from(std::mem::size_of_val(data))?;
	let mut buffer: Option<d3d11::ID3D11Buffer> = None;

	unsafe {
		device.CreateBuffer(
			&d3d11::D3D11_BUFFER_DESC {
				ByteWidth: size,
				Usage: d3d11::D3D11_USAGE_DEFAULT,
				BindFlags: bind_flags,
				CPUAccessFlags: 0,
				MiscFlags: 0,
				StructureByteStride: size_of::<T>().try_into()?,
			},
			Some(&d3d11::D3D11_SUBRESOURCE_DATA {
				pSysMem: data.as_ptr().cast::<std::ffi::c_void>(),
				SysMemPitch: 0,
				SysMemSlicePitch: 0,
			}),
			Some(&mut buffer),
		)?;
	}

	return Ok(buffer.expect("failed to create buffer"));
}
