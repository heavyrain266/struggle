//! Hardware Abstraction Layer
//!
//! A set of modules used to abstract d3d11 calls away from application

mod shared;

pub mod cx;
pub mod misc;

use windows::{
	Win32::{
		Foundation::{BOOL, HMODULE, HWND, RECT},
		Graphics::{
			Direct3D::{self as d3d, Fxc as fxc},
			Direct3D11 as d3d11, Dxgi as dxgi,
		},
	},
	core::{HSTRING, Interface, s},
};
