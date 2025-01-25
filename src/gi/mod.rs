//! Graphics Interface
//!
//! A set of modules used to interact with GPU.

pub(crate) mod adapter;
pub(crate) mod misc;

use windows::{
	core::{s, Interface, HSTRING},
	Win32::{
		Foundation::{BOOL, HMODULE, HWND},
		Graphics::{
			Direct3D::{self as d3d, Fxc as fxc},
			Direct3D11 as d3d11, Dxgi as dxgi,
		},
	},
};
