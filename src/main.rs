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
#![allow(non_snake_case)]

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

extern crate either;

extern crate confy;
extern crate serde;
extern crate serde_json;

extern crate image;
extern crate line_drawing;

extern crate rand;

mod sprite_sheet;
mod frame;

mod controls;
use crate::controls::*;

mod interface;
use crate::interface::*;

mod renderer;
use crate::renderer::*;

mod entity;

mod world;
use crate::world::*;

mod audio;
use crate::audio::*;

mod math;
mod animation;

mod collision;
use crate::collision::*;

mod timer;
mod text;
mod sound;
mod config;

use config::*;

/// Entry point for space invaders
/// 
/// Creates the hardware interface, populates the game world, and then enters the game loop
fn main() {
    env_logger::init();

    // create the hardware interface ... wgpu/pixels on desktop and 32bit for STM hardware (TODO)
    let (event_loop, mut interface) = create_interface("Space Invaders");
    
    // load config
    let mut config = Config::new();

    // create the initial state of the game world
    let mut world = initial_world_state(&config);
    
    // enter game loop
    event_loop.run(move |event, _, control_flow| {
        let current_state = world.get_current_state();

        // do we need to update the display
        if interface.render(&event) {
            // begin rendering, need by some backends
            interface.begin_draw();
            
            // render game if playing or paused
            if  current_state == GameState::Playing || current_state == GameState::Paused {
                renderer_system(&world, &mut interface);
            }
            // should we display the gameover message
            else if current_state == GameState::GameOver {
                renderer_gameover(&world, &mut interface);
            }
            // or otherwise might be the splash screen
            else if current_state == GameState::Splash {
                renderer_splash(&world, &mut interface);
            }

            // end redering, need to close drawing surfaces on some backends
            interface.end_draw();
        }

        let (should_exit, controls) = interface.handle_input(event);        

        // check if we should quit and exit if requested
        if should_exit {
            // fetch and store high score for next play
            config.udpate_highscore(world.get_high_score());
            config.store();
            *control_flow = ControlFlow::Exit;
            return;
        }
        
        // handle the state when game is in full swing
        if  current_state == GameState::Playing {
            world.play_music(world.get_current_bpm());
            // handle updates for player, alien, and ship components
            player_control_system(&mut world, controls);
            // handle movment update for all types of bullets
            bullet_control_system(&mut world);
            // handle movement updates for aliens
            alien_control_system(&mut world);
            // handle movment of UFO
            ship_control_system(&mut world);

            // handle bullet collisons, possible end game state reached on return...
            bullet_collision_system(&mut world);

            // handle the audio system
            audio_system(&world);

            // finally update the world to handle any internal changes
            world.update();
        }
        else if current_state == GameState::Splash {
            // showing spash screen and fire (space) is pressed
            if let Some(control) = controls {
                // start game
                if control.fire {
                    world.set_current_state(GameState::Playing);
                    // initalize ufo timer to something random
                    world.reset_ufo_timer();
                }
            }
        }

        // game over? 
        if current_state == GameState::GameOver {
            // pause any sounds that might be playing
            world.pause_music();
            world.pause_ufo();
            
            // is it time to move on?
            if world.has_game_over_timer_expired() {
                world.set_current_state(GameState::Splash);
                new_game(&mut world);
            }
            else {
                world.game_over_next();
            }
        }
        //move on to next level?
        else if current_state == GameState::NextLevel {
            world.pause_music();
            if world.has_next_level_timer_expired() {
                next_level(&mut world);
                world.set_current_state(GameState::Playing);
            }
        }

        interface.request_redraw();
    });
}