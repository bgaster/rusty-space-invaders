//! Description: 
//! 
//! Simple Framebuffer abstraction. Depending on if we are using Pixels at or Raylib
//! 
//! Copyright © 2020 Benedict Gaster. All rights reserved.

#[cfg(not(feature = "use-raylib"))]
mod frameinner {

    pub struct Frame<'a> {
        /// frame buffer data
        frame: &'a mut [u8],
        /// width of framebuffer
        width: u32,
        /// height of framebuffer
        height: u32,
    }

    impl<'a> Frame<'a> {
        pub fn new(frame: &'a mut [u8], width: u32, height: u32) -> Self {
            Frame {
                frame,
                width,
                height,
            }
        }

        pub fn put_pixel(&mut self, x: u32, y: u32, rgba: &[u8; 4]) {
            if x < self.width*4 && y < self.height*4 {
                let offset = (x+y*self.width*4) as usize;
                self.frame[offset..offset + 4].copy_from_slice(rgba);
            } 
        }
    }

}

#[cfg(feature = "use-raylib")]
mod frameinner {
    pub struct Frame<'a> {
        /// width of framebuffer
        width: u32,
        /// height of framebuffer
        height: u32,
        _marker: ::std::marker::PhantomData<&'a mut ()>
    }

    impl<'a> Frame<'a> {
        pub fn new(width: u32, height: u32) -> Self {
            Frame {
                width,
                height,
                _marker: ::std::marker::PhantomData,
            }
        }

        pub fn put_pixel(&mut self, x: u32, y: u32, rgba: &[u8; 4]) {
            if x < self.width*4 && y < self.height*4 {
            } 
        }
    }
}

// export whichever frame was selected, depending on feature
pub use frameinner::*;