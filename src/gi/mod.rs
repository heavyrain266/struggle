//! Graphics Interface
//!
//! A set of modules used to interact with GPU.

mod buffers;

pub(crate) mod context;
pub(crate) mod misc;

use windows::{
	Win32::{
		Foundation::{BOOL, HMODULE, HWND},
		Graphics::{
			Direct3D::{self as d3d, Fxc as fxc},
			Direct3D11 as d3d11, Dxgi as dxgi,
		},
	},
	core::{HSTRING, Interface, s},
};
