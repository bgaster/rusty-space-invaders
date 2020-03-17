//! Description: 
//! 
//! Hardware interface abstraction. The particular choice of hardware abstraction is done via
//! specific features. Curreltly the following are supported:
//! 
//!     desktop (wgpu)             - macos, windows, Non Raspberry Pi Linux distros 
//!     use-raylib (OpenGL ES 2.1) - Raspberry Pi 3.x, this requires Raylib and is aimed at TFT screens (but should 
//!                                  work for X too)
//!     thirtytwo-blit             - 32Blit
//!  
//! Copyright © 2020 Benedict Gaster. All rights reserved.

#[cfg(not(feature = "use-raylib"))]
include!("interface_desktop.in");

#[cfg(feature = "use-raylib")]
include!("interface_raylib.in");