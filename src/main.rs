//! Description: 
//!   An emulation of the original space invaders. The behaviour follows that described at:
//! 
//!         Chris Cantrell
//!         https://computerarcheology.com/Arcade/SpaceInvaders/
//!         
//! It is not an direct copy, in particular, the timing is similar, but I've not made any real effort to match
//! it percisely. It is not implemented as per the original game, using the 2 screen interrupts and so on, insead
//! it uses a simply timer based for the main alien swarm and the other elements, e.g. bullets, just run at their 
//! own rate, both for animations, and for movement. It does not use an ECS, which if I was doing anything more 
//! complicated I should have. Next project I plan to, but this was just a small few day project while I sat around
//! complaining about a horrid tooth ache, which has now been fixed with a root canal :-)
//! 
//! It is worth noting that the main goal of the project is to port it to the 32blit, a small 32-bit MCU based 
//! retro games console, which I backed on Kickstarter and should be arriving soon. More details of this here:
//! 
//!     https://www.kickstarter.com/projects/pimoroni/32blit-retro-inspired-handheld-with-open-source-fi
//! 
//! See the README.md for move information.
//! 
//! Copyright © 2020 Benedict Gaster. All rights reserved.

// #![deny(clippy::all)]
#![forbid(unsafe_code)]

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate either;

extern crate serde;
extern crate serde_json;

extern crate image;
extern crate line_drawing;

extern crate rand;

use image::{Rgba};

use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use gilrs::{Button, Gilrs};

use log::{info, trace, warn};

mod sprite_sheet;
use crate::sprite_sheet::*;

mod frame;
use crate::frame::*;

mod controls;
use crate::controls::*;

mod interface;
use crate::interface::*;

mod renderer;
use crate::renderer::*;

mod entity;
use crate::entity::*;

mod world;
use crate::world::*;

mod audio;
use crate::audio::*;

mod math;
mod animation;

mod collision;
use crate::collision::*;

mod timer;

const BOX_SIZE: i16 = 64;

fn main() -> Result<(), Error> {
    env_logger::init();

    // create the hardware interface ... wgpu/pixels on desktop and 32bit for STM hardware (TODO)
    let (event_loop, mut interface) = create_interface("Space Invaders");
    
    let mut world = initial_world_state();

    world.reset_lag();
    event_loop.run(move |event, _, control_flow| {
        if interface.render(&event) {
            renderer_system(&world, &mut interface);
        }
        // TODO: audio system

        let (should_exit, controls) = interface.handle_input(event);        

        // check if we should quit and exit if requested
        if should_exit {
            *control_flow = ControlFlow::Exit;
            return;
        }

        

        // handle updates for player, alien, and ship components
        player_control_system(&mut world, controls);
        bullet_control_system(&mut world);

        alien_control_system(&mut world);
        ship_control_system(&world);

        // handle bullet collisons
        bullet_collision_system(&mut world);

        audio_system(&world);

        // finally update the world to handle any internal changes
        world.update();

        interface.request_redraw();
    });
}