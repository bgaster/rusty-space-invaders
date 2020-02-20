
use std::cmp::min;
use either::*;
use line_drawing::{Bresenham};

use crate::interface::*;
use crate::frame::*;
use crate::interface::*;
use crate::entity::*;
use crate::world::*;
use crate::math::*;

lazy_static! {
    static ref SCREEN_LINE: Rect = Rect::new(Point::new(0,362), Size::new(Interface::get_width(), 2));
}

pub fn renderer_system(world: &World, interface: &mut Interface) {

    interface.clear_framebuffer([0x0,0x0,0x0,0xFF]);
        
    let p = interface.pixels.get_frame();
    let mut frame = Frame {
        frame: p,
        width: Interface::get_width(),
        height: Interface::get_height(),
    };

    // get ref to sprite sheet used to render sprites and animations
    let sheet = world.get_sprite_sheet();

    // first draw the player
    if let Some(entity) = world.get_entity(world.get_player()) {
        if let Entity::Player(player) = entity {
            if player.lives_remaining != 0 && !world.get_player_died() {
                player.sprite.render(player.position.x, player.position.y, sheet, &mut frame);
            }

            // draw play bullet if in flight
            if player.bullet.bullet_mode == BulletMode::InFlight {
                if let Left(sprite) = player.bullet.sprite.clone() {
                    sprite.render(player.bullet.position.x, player.bullet.position.y, sheet, &mut frame);
                }
            }

            // draw any lives left
            for i in  0..player.lives_remaining as u32-1 {
                player.sprite.render(
                    PLAYER_LIVES_TOP_LEFT_X_START_POSITION + (player.sprite.width + 100)*i, 
                    PLAYER_LIVES_TOP_LEFT_Y_START_POSITION, 
                    sheet, 
                    &mut frame);
            }
        }
    }

    // draw aliens
    for index in world.get_aliens() {
        if let Some(entity) = world.get_entity(index) {
            if let Entity::Alien(alien) = entity {
                // only draw alive aliens
                if alien.is_alive {
                    alien.animation.render(alien.position, sheet, &mut frame);
                }
            }
        }
    }

    // draw bullets
    
    // alien bullets that are in flight
    for index in world.get_alien_bullets().iter() {
        if let Some(entity) = world.get_entity(*index) {
            if let Entity::Bullet(bullet) = entity {
                if bullet.bullet_mode == BulletMode::InFlight {
                    match bullet.sprite.clone() {
                        Left(sprite) =>  {
                            sprite.render(bullet.position.x, bullet.position.y, sheet, &mut frame);
                        },
                        Right(animation) =>  {
                            animation.render(bullet.position, sheet, &mut frame);
                        },
                    }
                }
            }
        }
    }

    for index in world.get_explosions() {
        if let Some(entity) = world.get_entity(index) {
            if let Entity::BulletExplosion(explosion) = entity {
                match explosion.sprite.clone() {
                    Left(sprite) =>  {
                        sprite.render(explosion.position.x, explosion.position.y, sheet, &mut frame);
                    },
                    Right(animation) =>  {
                        animation.render(explosion.position, sheet, &mut frame);
                    },
                }
                //explosion.sprite.render(explosion.position.x, explosion.position.y, sheet, &mut frame);
            }
        }
    }

    // TODO: handle ship

    // draw line at bottom of screen
    //fill_rect(Point::new(0, 360), Point::new(Interface::get_width(), 362), [0x28, 0xcf, 0x28, 0xFF], frame.frame );

    fill_rect(
        *SCREEN_LINE, 
        [0x28, 0xcf, 0x28, 0xFF], 
        frame.frame );

    interface.draw_call();
}

// utility functions

/// Draw a line to the pixel buffer using Bresenham's algorithm.
pub fn line(p1: Point, p2: Point, color: [u8; 4], screen: &mut [u8]) {
    let p1 = (p1.x as i64, p1.y as i64);
    let p2 = (p2.x as i64, p2.y as i64);

    for (x, y) in Bresenham::new(p1, p2) {
        let x = min(x as usize, Interface::get_width() as usize - 1);
        let y = min(y as usize, Interface::get_height() as usize - 1);
        let i = x * 4 + y * Interface::get_width() as usize * 4;

        screen[i..i + 4].copy_from_slice(&color);
    }
}

/// Draw a rectangle to the pixel buffer using two points in opposite corners.
pub fn rect(p1: Point, p2: Point, color: [u8; 4], screen: &mut [u8]) {
    let p2 = Point::new(p2.x - 1, p2.y - 1);
    let p3 = Point::new(p1.x, p2.y);
    let p4 = Point::new(p2.x, p1.y);

    line(p1, p3, color, screen);
    line(p3, p2, color, screen);
    line(p2, p4, color, screen);
    line(p4, p1, color, screen);
}


pub fn fill_rect(rect: Rect, color: [u8; 4], screen: &mut [u8]) {
    for y in 0..=rect.height() {
        line(Point::new(rect.min_x(), rect.min_y() + y), Point::new(rect.max_x(), rect.min_y() + y), color, screen);
    }
}