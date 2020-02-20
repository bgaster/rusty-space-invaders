//! Description: 
//! 
//! 
//! Copyright © 2020 Benedict Gaster. All rights reserved.

pub struct Frame<'a> {
    /// frame buffer data
    pub frame: &'a mut [u8],
    /// width of framebuffer
    pub width: u32,
    /// height of framebuffer
    pub height: u32,
}