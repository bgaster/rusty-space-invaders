//! Description: 
//! 
//! 
//! Copyright © 2020 Benedict Gaster. All rights reserved.

use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{WindowBuilder, Window};
use winit_input_helper::WinitInputHelper;
use gilrs::{Button, Gilrs};
use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};

use crate::controls::*;

const WIDTH: u32 = 480;
const HEIGHT: u32 = 400;

pub struct Interface {
    pub window: Window,
    pub hidpi_factor: f64,
    pub input: WinitInputHelper,
    pub pixels: Pixels,
}

impl Interface {
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn handle_input<'a>(&mut self, event: Event<'a, ()>) -> (bool, Option<Controls>) {

        // handle any redraw events (TODO: not sure this makes sense here!!)
        
        if self.input.update(event) {
            if self.input.key_pressed(VirtualKeyCode::Escape) || self.input.quit() {
                (true, None)
            }
            else {
                let controls = {
                    // Keyboard controls
                    let left = self.input.key_held(VirtualKeyCode::Left);
                    let right = self.input.key_held(VirtualKeyCode::Right);
                    let fire = self.input.key_pressed(VirtualKeyCode::Space);

                    let direction = if left {
                        Direction::Left
                    } else if right {
                        Direction::Right
                    } else {
                        Direction::Still
                    };

                    Controls { direction, fire }
                };
        
                // Adjust high DPI factor
                if let Some(factor) = self.input.scale_factor_changed() {
                    self.hidpi_factor = factor;
                }
        
                // Resize the window
                if let Some(size) = self.input.window_resized() {
                    //let size = size.to_physical(hidpi_factor);
                    let width = size.width; // .round() as u32;
                    let height = size.height; //.round() as u32;

                    self.pixels.resize(width, height);
                }

                (false, Some(controls))
            }
        }
        else {
            (false, None)
        }
    }
    
    pub fn clear_framebuffer(&mut self, color: [u8;4]) {
        let fb = self.pixels.get_frame();

        for pixel in fb.chunks_exact_mut(4) {
            pixel.copy_from_slice(&color);
        }
    }

    pub fn get_framebuffer(&mut self) -> &mut [u8] {
        self.pixels.get_frame()
    }

    pub fn render<'a>(&self, event: &Event<'a, ()>)-> bool {
        if let Event::RedrawRequested(_) = event {
            return true;
        }
        false
    }

    pub fn draw_call(&mut self) {
        self.pixels.render();
    }

    #[inline]
    pub fn get_width() -> u32 {
        WIDTH
    }

    #[inline]
    pub fn get_height() -> u32 {
        HEIGHT
    }
}

pub fn create_interface(title: &str) -> (EventLoop<()>, Interface) { 
        let event_loop = EventLoop::new();
            let input = WinitInputHelper::new();
            let window = {
                let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
                WindowBuilder::new()
                    .with_title(title)
                    .with_inner_size(size)
                    .with_min_inner_size(size)
                    .build(&event_loop)
                    .unwrap()
            };
        let hidpi_factor = window.scale_factor();


        let pixels = {
            let surface = Surface::create(&window);
            let surface_texture = SurfaceTexture::new(WIDTH, HEIGHT, surface);
            Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
        };

        (event_loop,
         Interface {
            window,
            hidpi_factor,
            input,
            pixels,
         })
}