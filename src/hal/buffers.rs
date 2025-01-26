//! Buffers
//!
//! A module used to create and manage buffers

//use super::d3d11;
//
//#[repr(C)]
//#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
//pub(super) struct Vertex {
//	position: [f32; 3],
//	color: [f32; 4],
//}
//
//impl Vertex {
//	pub(super) const fn new(position: [f32; 3], color: [f32; 4]) -> Self {
//		Self { position, color }
//	}
//}
//
//pub(super) fn vertex_buffer(
//	device: d3d11::ID3D11Device, vertices: &[Vertex],
//) -> Result<d3d11::ID3D11Buffer, windows::core::Error> {
//	let buffer_desc: d3d11::D3D11_BUFFER_DESC = d3d11::D3D11_BUFFER_DESC {
//		ByteWidth: (std::mem::size_of::<Vertex>() * vertices.len()) as u32,
//		Usage: d3d11::D3D11_USAGE_DEFAULT,
//		BindFlags: 1,
//		CPUAccessFlags: 0,
//		MiscFlags: 0,
//		StructureByteStride: std::mem::size_of::<Vertex>() as u32,
//	};
//
//	let subresource: d3d11::D3D11_SUBRESOURCE_DATA = d3d11::D3D11_SUBRESOURCE_DATA {
//		pSysMem: vertices.as_ptr() as *const std::ffi::c_void,
//		SysMemPitch: 0,
//		SysMemSlicePitch: 0,
//	};
//
//	unsafe {
//		let mut vertex_buffer: Option<d3d11::ID3D11Buffer> = None;
//		device.CreateBuffer(&buffer_desc, Some(&subresource), Some(&mut vertex_buffer))?;
//
//		Ok(vertex_buffer.unwrap())
//	}
//}
