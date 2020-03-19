//! Description: 
//! 
//! Handle general control systems for player, aliens, bullets, and, of course, the ship
//! 
//! Copyright © 2020 Benedict Gaster. All rights reserved.
//! 

use either::*;

use crate::entity::*;
use crate::world::*;
use crate::math::*;

/// Player control inputs.
#[derive(Debug)]
pub struct Controls {
    /// Move the player.
    pub direction: Direction,
    /// Shoot the cannon.
    pub fire: bool,
}

/// The player can only move left or right, but can also be stationary.
#[derive(Debug)]
pub enum Direction {
    /// Do not move the player.
    Still,
    /// Move to the left.
    Left,
    /// Move to the right.
    Right,
}

impl Default for Controls {
    fn default() -> Controls {
        Controls {
            direction: Direction::default(),
            fire: false,
        }
    }
}

impl Default for Direction {
    fn default() -> Direction {
        Direction::Still
    }
}

// player control system, control movement of player and firing
pub fn player_control_system(world: &mut World, controls: Option<Controls>) {

    // is the player in the process of dying (we assume that they still have lives, checked elsewhere)
    if world.get_player_died() {
        // if timer has expired next player life is respawned and game continues
        if world.has_player_died_timer_expired() {
            *world.get_mut_player_died() = false;
        }
        // otherwise no control updates happen for player
        else {
            return;
        }
    }

    // handle input
    if let Some(controls) = controls {
        // handle input system
        let movement = match controls.direction {
            Direction::Left => -World::player_movement(),
            Direction::Right => World::player_movement(),
            Direction::Still => 0
        };

        let bounds = world.get_bounds();
        let bullet_speed = world.get_player_bullet_speed();
        let mut bullet_explosion = None;

        let mut fire_sound = false;
        if let Some(entity) = world.get_mut_entity(world.get_player()) {
            if let Entity::Player(player) = entity {

                if player.bullet.bullet_mode == BulletMode::Fire {
                    // player pressed fire and a they do not already have a bullet in play, then generate one
                    // we do this before updating the player's movement...
                    if controls.fire {
                        player.bullet.bullet_mode = BulletMode::InFlight;

                        player.bullet.position = Point::new(
                            player.position.x + player.bounding_box.width()/2 + 30,
                            ((player.position.y as i32) - player.bullet.bounding_box.height() as i32) as u32);
                        
                        fire_sound = true;
                    }
                }
                // animate player bullet if play 
                else {
                    player.bullet.position.y = (player.bullet.position.y as i32 - bullet_speed) as u32;
                    if player.bullet.position.y <= bounds.min_y() {
                        bullet_explosion = Some(player.bullet.position);
                        player.bullet.bullet_mode = BulletMode::Fire;
                    }
                }

                let x = (player.position.x as i32 + movement) as u32;
                if x >= bounds.min_x() && x+player.sprite.width*4 <= bounds.max_x() {
                    player.position = Point::new(x, player.position.y)
                }
            }
        }

        // now that we have the world back we can add a bullet explosion, if necessary
        if let Some(position) = bullet_explosion {
            let bullet_explosion_sprite = world.get_player_bullet_explosion_sprite();
            world.add_explosion(
                Entity::BulletExplosion(
                    BulletExplosion::new(
                        position, 
                        bullet_explosion_sprite, 
                        world.get_bullet_explosion_time() as i32)));
        }

        // finally if we need to play the player fire sound, then do
        if fire_sound {
            world.play_player_shot();
        }
    }
}

// handle bullet control, this can mean moving alien bullets, player bullet movement is handled by
// the player, and all explosions
pub fn bullet_control_system(world: &mut World) {

    // is the player in the process of dying, then no updates take place, except for any explosion animations
    if world.get_player_died() {
        // step any bullet explosions
        for index in 0..world.number_explosions() {
            if let Some(entity) = world.get_mut_entity(world.get_explosion_index(index)) {
                if let Entity::BulletExplosion(explosion) = entity {
                    if let Right(animation) = &mut explosion.sprite {
                        animation.step();
                    }
                }
            }
        }
        return;
    }

    // get the players x position, used for targeted bullet
    let mut player_x_position = 0; 
    if let Some(entity) = world.get_mut_entity(world.get_player()) {
        if let Entity::Player(player) = entity {
            player_x_position = player.position.x;
        }
    }

    // first we step any alien bullets that are in flight
    if world.has_animate_alien_bullet_timer_expired() {
        for index in world.get_alien_bullets().iter() {
            if let Some(entity) = world.get_mut_entity(*index) {
                if let Entity::Bullet(bullet) = entity {
                    if bullet.bullet_mode == BulletMode::InFlight {
                        bullet.position.y = 
                            (bullet.position.y as i32 + World::get_alien_bullet_initial_speed() as i32) as u32;
                        if let Right(animation) = &mut bullet.sprite {
                            animation.step();
                        }
                    }
                }
            }
        }

        world.reset_animate_alien_bullet_timer();
    }

    // add an alien bullet?
    if world.has_alien_bullet_timer_expired() {
        // what type of bullet should we create?
        let next = world.get_next_alien_bullet_type();

        // first get the position of alien that will drop bullet
        let mut alien_position = Point::new(player_x_position,0);
        match next {
            AlienBulletType::Rolling => {
                // find alien that is closet to the player
                for column in 0..world.get_number_alien_columns() {
                    if let Some(index) = world.lowest_alive_alien_in_column(column) {
                        if let Some(entity) = world.get_mut_entity(index) {
                            if let Entity::Alien(alien) = entity {
                                if player_x_position < alien.position.x {
                                    // handle the case when player is to the left of aliens
                                    //if column == 0 {
                                        alien_position = alien.position;
                                    //}
                                    // centre the bullet
                                    alien_position.x += alien.bounding_box.size.width ;
                                    alien_position.y += alien.bounding_box.size.height ;
                                    break;
                                }
                                alien_position = alien.position;
                                if player_x_position*4 >= alien.position.x && 
                                   player_x_position*4 <= alien.position.x + alien.bounding_box.size.width {
                                    // centre the bullet
                                    alien_position.x += alien.bounding_box.size.width;
                                    alien_position.y += alien.bounding_box.size.height;
                                    break;
                                }
                            }
                        }       
                    }
                }
            },
            // for now we will randomly generate a column, but we might want to follow the actual game a bit closer
            // by simply using a look up table...
            AlienBulletType::Plunger | AlienBulletType::Squiggly => {
                // loop until we find a column with an alien
                // NOTE: this only terminates if NOT ALL aliens are dead, make sure this is not the case :-)
                loop {
                    let column = world.gen_rand_column();
                    if let Some(index) = world.lowest_alive_alien_in_column(column) {
                        if let Some(entity) = world.get_mut_entity(index) {
                            if let Entity::Alien(alien) = entity {
                                alien_position = alien.position;
                                alien_position.x += alien.bounding_box.size.width ;
                                alien_position.y += alien.bounding_box.size.height ;
                                break;
                            }
                        }
                    }
                }
            },
        };

        // find the entity for our bullet
        let entity = match next {
            AlienBulletType::Rolling => world.get_mut_entity(world.get_alien_rolling()),
            AlienBulletType::Plunger => world.get_mut_entity(world.get_alien_plunger()),
            AlienBulletType::Squiggly => world.get_mut_entity(world.get_alien_squiggly()),
        };

        if let Some(Entity::Bullet(bullet)) = entity {
            // bullet must be in fire mode to fire
            if bullet.bullet_mode == BulletMode::Fire {
                // tracking bullet?
                let bullet_position = alien_position;                    
                bullet.position = bullet_position;
                bullet.bullet_mode = BulletMode::InFlight;
            } 
        }

        // next time we do a different bullet type
        world.inc_next_bullet_type();

        // finally reset timer so we can add another bullet
        world.reset_alien_bullet_timer();
    }

    let mut indexes = vec![];
    for index in 0..world.number_explosions() {
        if let Some(entity) = world.get_mut_entity(world.get_explosion_index(index)) {
            if let Entity::BulletExplosion(explosion) = entity {
                if explosion.framecount == 0 {
                    indexes.push(index);
                }
                else {
                    explosion.framecount -= 1;
                    if let Right(animation) = &mut explosion.sprite {
                        animation.step();
                    }
                }
            }
        }
    }

    for index in &indexes {
        world.delete_explosion(*index);
    }
}

// alien control system, control movement of swarm
pub fn alien_control_system(world: &mut World) {

    // is the player in the process of dying, then no updates take place
    if world.get_player_died() {
        return;
    }

    let bounding_width = 24;
    // step animations 
    let elasped_time = world.get_lag();

    let step = elasped_time >= world.get_alien_swarm_speed();
    let step_ani = elasped_time.as_millis() % 16 == 0;
    let top_left_pos = world.get_alien_swarm_top_left_position();
    // we now account for empty left hand columns
    // we keep these seperate as our reference point is the whole swam, live or dead
    let actual_top_left_pos = 
        Point::new(
            top_left_pos.x + (world.get_alien_spacing_horz())*world.left_most_alien_column(), 
            top_left_pos.y);

    let mut direction = world.get_alien_swarm_direction();
    let speed = world.get_alien_speed();

    let extra: u32 = ((world.get_number_alien_columns() as u32 - 1) - world.right_most_alien_column() as u32) * 6;
    let top_right_pos = 
        top_left_pos.x + world.right_most_alien_column() as u32 * 
            (world.get_alien_spacing_horz() + bounding_width ) + extra; //- 2*world.get_alien_spacing_horz();

    let bounds = world.get_bounds();
    // resolve alien swarm next move, i.e. change direction or continue on current path
    let mut step_down: u32 = 0;

    // if world.right_most_alien_column() == 1 {
    //     println!("{} {}", top_right_pos, bounds.max_x());
    // }

    if step && top_right_pos >= bounds.max_x() - world.get_alien_speed() as u32 && direction == 1 {
        *world.get_mut_alien_swarm_direction() = -1;
        direction = -1;
        step_down = world.get_alien_step_down();
    }
    else if step && actual_top_left_pos.x  <= bounds.min_x() + world.get_alien_speed() as u32 && direction == -1 {
        *world.get_mut_alien_swarm_direction() = 1;
        direction = 1;
        step_down = world.get_alien_step_down();
    }

    // if step {
    //     println!(">>>");
    // }
    for alien_index in 0..world.get_number_aliens() {
        if let Some(entity) = world.get_mut_entity(world.get_alien(alien_index)) {
            if let Entity::Alien(alien) = entity {
                // step internal animation, if necessary 
                if step_ani {
                    alien.animation.step();
                }

                // step each alien within swarm, if necessary
                if step && alien.is_alive {
                    let x = (alien.position.x as i32 + speed * direction) as u32;
                    let y = alien.position.y + step_down;
                    alien.position = Point::new(x, y);
                    //println!("{:?}", alien.position);
                }
            }
        }
    }
    // if step {  
    //     println!("<<<");
    // }

    // update swam overall position for next time
    if step {
        let pos = world.get_mut_alien_swarm_top_left_postion();
        pos.x = (pos.x as i32 + speed * direction) as u32;

        world.reset_lag();
    }
}

pub fn ship_control_system(world: &mut World) {
    // if the player in the process of dying, then no updates take place
    if world.get_player_died() {
        return;
    }

    let bounds = world.get_bounds();
    let ufo_timer_expired = world.has_ufo_timer_expired();
    let mut reset_timer = false;
    let mut play_effect = false;
    if let Some(entity) = world.get_mut_entity(world.get_ship()) {
        if let Entity::Ship(ship) = entity {
            if ship.is_alive {
                ship.position.x += World::ship_movement();

                // has ship make it to the right edge of the window?
                if ship.position.x + ship.bounding_box.size.width >= bounds.max_x() {
                    ship.is_alive = false;
                    reset_timer = true;
                }
            }
            else if ufo_timer_expired {
                ship.is_alive = true;
                play_effect = true;
                ship.position = Point::new(UFO_START_X_START_POSITION, UFO_START_Y_START_POSITION);
            }
        }
    }

    // if the UFO got the edge of the screen we need to reset its timer and stop sound effect
    if reset_timer {
        world.reset_ufo_timer();
        world.pause_ufo();
    }
    // if UFO entering the screen enable ufo sound effect
    else if play_effect {
        world.play_ufo();
    }
}


/// is it game over, i.e. player has no lives left?
pub fn is_game_over(world: &World) -> bool {
    if let Some(entity) = world.get_entity(world.get_player()) {
        if let Entity::Player(player) = entity {
            return player.lives_remaining == 0;
        }
    }

    return false;
}
