//! Description: 
//! 
//! Handle collisions, which for space invaders means handle bullets hitting things :-)
//! 
//! Copyright © 2020 Benedict Gaster. All rights reserved.
//! 

use crate::entity::*;
use crate::world::*;
use crate::math::*;
use crate::sprite_sheet::{SpriteMask};

/// apply a mask to another mask, clearing any pixels that are set in both masks, when overlapped
/// 
/// # Arguments
/// 
/// * `x` - x position to apply mask
/// * `y` - y position to apply mask
/// * `apply_mask` - mask to apply
/// * `mask` - mask that mask is applied to, this is side effected if masks overlap set pixels
fn apply_mask(x: usize, y: usize, apply_mask: &SpriteMask, mask: &mut SpriteMask) {
    for yy in 0..apply_mask.len() {
        // don't go of the bottom of sprite
        if y+yy >= mask.len() {
            return;
        }
        else {
            for xx in 0..apply_mask[0].len() {
                // don't go of right edge of sprite
                if x+xx >= mask[0].len() {
                    break;
                }
                if apply_mask[yy][xx] != 0 {
                    mask[y+yy][x+xx] = 0;   
                }
            }
        }
    }
}

/// handle player or alien bullet collisions with barriers, returns true in case of collison, otherwise false
/// 
/// # Arguments
///
/// * `is_alien`       - A boolean determing if the bullet is from an alien or player
/// * `bounding_box`   - Bounding box of bullet
/// * `explosion_mask` - mask applied to barrier mask, if bullet collides with any remaining pixels in barrier 
/// *  `barriers_info` - Info related to all barriers that bullet could collide with. If a bullet collides with 
///                      a barrier the barriers mask is updated to refect the hit, providing more of a passage 
///                      through.
fn collides_with_barrier(
    is_alien: bool,
    bounding_box: &Rect, 
    explosion_mask: &SpriteMask,
    barriers_info: &mut Vec<(EntityIndex, Rect, SpriteMask)>) -> bool {

    for i in barriers_info {        
        // first check intersection for barrier bounding box
        if i.1.intersects(bounding_box) {
            // now we need to dig deeper and check if the bullet intersects with the barrier sprite mask
            let bullet_pos_x = if i.1.origin.x > bounding_box.origin.x {
                i.1.origin.x - bounding_box.origin.x 
            } else {
                bounding_box.origin.x - i.1.origin.x
            };

            let barrier_height = i.1.size.height;
            let barrier_width  = i.1.size.width>>2;  // TODO: actally address the multiple of 4 issue at its root 
            let bullet_width   = bounding_box.size.width as usize >> 2;
            let bpos = (bullet_pos_x as usize) >> 2;
            let width = if bpos + bullet_width as usize > barrier_width as usize {
                barrier_width as usize
            }
            else {
                bpos + bullet_width as usize
            };

            if is_alien {
                // handle alien bullet, which will becoming from top down
                let mask = &mut i.2;
                for y in 0..barrier_height as usize {
                    for x in (bullet_pos_x as usize) >> 2..width as usize {
                        if mask[y][x] == 1 {
                            apply_mask(x-2, y, explosion_mask, mask);
                            return true;
                        }
                    }
                }
            }
            else {
                // handle player bullet, which will becoming from bottom up
                let mask = &mut i.2;
                let bpos_y = (i.1.origin.y + i.1.size.height) - bounding_box.origin.y;
                // iterate from bottom of barrier when the bullet hit, moving up until we find a set pixel to distory
                for y in (0..=barrier_height as usize).rev() {
                    for x in bpos..width { 
                        // is barrier pixel set
                        if mask[y][x] == 1 {
                            // we move up to apply mask and clamp if at top of barrier mask
                            let y_clamped = 
                                if y as i32 - explosion_mask.len() as i32 > 0 {
                                    (y - explosion_mask.len())+1
                                } 
                                else {
                                    0
                                };
                            apply_mask(bpos, y_clamped, explosion_mask, mask);
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// handle any alien and player bullet collisions, with barriers, the player, and aliens
/// 
/// # Arguments
/// 
/// * `world` - The game world
pub fn bullet_collision_system(world: &mut World) {

    // get the players position, used to check again alien bullets
    let mut player_position = Point::default(); 
    let mut player_bounding_box = Rect::default();
    let mut player_bullet_bounding_box = Rect::default();
    let mut player_bullet_in_flight = false;

    if let Some(entity) = world.get_entity(world.get_player()) {
        if let Entity::Player(player) = entity {
            player_position = player.position;
            player_bounding_box = player.bounding_box;
            player_bounding_box.origin = player_position;

            if player.bullet.bullet_mode == BulletMode::InFlight {
                player_bullet_bounding_box = player.bullet.get_bounding_box();
                player_bounding_box.origin = player.bullet.position;
                player_bullet_bounding_box.size = 
                    Size::new(player_bullet_bounding_box.size.width, player_bullet_bounding_box.size.height);
                player_bullet_in_flight = true;
            }
        }
    }

    // get barrier info
    let mut barriers = vec![];
    for index in world.get_barriers() {
        if let Some(entity) = world.get_entity(index) {
            if let Entity::Barrier(barrier) = entity {
                barriers.push(
                    (index, 
                     Rect::new(
                         barrier.bounding_box.origin, 
                         Size::new(barrier.bounding_box.size.width*4, barrier.bounding_box.size.height)),
                     barrier.mask.clone()));
            }
        }
    }

    let mut player_killed = false;
    let mut player_bullet_killed = false;
    let mut explosions: Vec<BulletExplosion> = vec![];
    let bullet_explosion_sprite = world.get_alien_bullet_explosion_sprite();
    let shield_bullet_explosion_mask = world.get_shield_bullet_explosion_mask();
    let mut barrier_update = false;
    // handle alien bullet collisions
    for index in world.get_alien_bullets().iter() {
        if let Some(entity) = world.get_mut_entity(*index) {
            if let Entity::Bullet(bullet) = entity {
                if bullet.bullet_mode == BulletMode::InFlight {
                    
                    // check for colision with player
                    let mut bullet_bounding_box = bullet.bounding_box;
                    bullet_bounding_box.origin = bullet.position;

                    // first check if collides with barrier
                    if collides_with_barrier(
                        true,
                        &bullet_bounding_box, 
                        &shield_bullet_explosion_mask, 
                        &mut barriers) {
                        barrier_update = true;
                        bullet.bullet_mode = BulletMode::Fire;
                    }
                    else if bullet.position.x >= player_position.x && 
                       bullet.position.x <= player_position.x + player_bounding_box.size.width*4 && // hmm need to fix...
                       bullet.position.y >= player_position.y && 
                       bullet.position.y <= player_position.y + player_bounding_box.size.height {
                        bullet.bullet_mode = BulletMode::Fire;
                        player_killed = true; // handle player death below
                    }
                    // check for colision with ground
                    else if bullet.position.y + bullet.bounding_box.size.height >= World::get_ground().origin.y {  
                        bullet.bullet_mode = BulletMode::Fire;
                        explosions.push(BulletExplosion::new(
                            bullet.position, 
                            bullet_explosion_sprite.clone(), 
                            world.get_bullet_explosion_time() as i32));
                    }
                    // check to see if it has hit the player's bullet
                    else if player_bullet_in_flight && bullet_bounding_box.intersects(&player_bullet_bounding_box) {
                        bullet.bullet_mode = BulletMode::Fire;
                        explosions.push(BulletExplosion::new(
                            bullet.position, 
                            world.get_alien_bullet_explosiion_with_player_bullet(), 
                            world.get_bullet_explosion_time() as i32));
                        player_bullet_killed = true;
                        
                    }
                }
            }
        }
    } 

    // now handle a player death
    if player_killed || player_bullet_killed {
        if let Some(entity) = world.get_mut_entity(world.get_player()) {
            if let Entity::Player(player) = entity {
                // reload bullet if killed by alien  bullet
                if player_bullet_killed {
                    player.bullet.bullet_mode = BulletMode::Fire;   
                }

                // kill player if killed by alien bullet
                if player_killed {
                    let pos = player.position;
                    player.position = World::player_start_position();
                    player.lives_remaining -= 1;

                    // set playing state to game over, if no lives left
                    if player.lives_remaining == 0 {
                        world.set_current_state(GameState::GameOver);    
                    }

                    explosions.push(BulletExplosion::new(
                        pos, 
                        world.get_player_explosion_sprite().clone(), 
                        world.get_bullet_explosion_time() as i32));

                    // finally reset the player killed timer to delay the gameplay for a moment
                    world.reset_player_died_timer();
                    *world.get_mut_player_died() = true;

                    // finally, play player explosion
                    world.play_player_explosion();
                }
            }
        }
    }

    // add any explosions to the world
    for e in explosions {
        world.add_explosion(Entity::BulletExplosion(e));
    }

    // player bullet collision with aliens 
    // we probably should do a quick bounding box around all aliens, but for now just test them all

    let mut bounding_box = None;
    if let Some(entity) = world.get_mut_entity(world.get_player()) {
        if let Entity::Player(player) = entity {
            if player.bullet.bullet_mode == BulletMode::InFlight {
                // first check if collides with barrier
                if collides_with_barrier(
                    false,
                    &player_bullet_bounding_box, 
                    &shield_bullet_explosion_mask,
                    &mut barriers) {
                    barrier_update  = true;
                    player.bullet.bullet_mode = BulletMode::Fire;
                }
                else {
                    bounding_box = Some(player.bullet.get_bounding_box());
                }
            }
        }
    }

    let mut alien_index = 0;
    let mut player_points_inc = 0;
    if let Some(bullet_bounding_box) = bounding_box {
        bounding_box = None; // assume we don't hit
        for index in 0..world.get_number_aliens() {
            if let Some(entity) = world.get_mut_entity(world.get_alien(index)) {
                if let Entity::Alien(alien) = entity {
                    if alien.is_alive {
                        let alien_bounding_box = alien.get_bounding_box();
                        // do they intersect ?
                        if alien_bounding_box.intersects(&bullet_bounding_box) {
                            // kill alien
                            alien.is_alive = false;
                            // set so we can update player once we have world ownership back
                            // and add explosion
                            bounding_box = Some(alien_bounding_box);
                            // track alien index so we can remove it from column count
                            alien_index = index;
                            player_points_inc = alien.points;
                            break;
                        }
                    }
                }
            }
        }
    }

    // update barrier mask if bullet collided
    if barrier_update {
        for i in barriers {
            if let Some(entity) = world.get_mut_entity(i.0) {
                if let Entity::Barrier(barrier) = entity {
                    barrier.mask = i.2;
                }
            }
        }
    }

    
    //if update_player_bullet {
    if let Some(alien_bounding_box) = bounding_box {

        // add explosion
        let bullet_explosion_sprite = world.get_player_alien_bullet_explosion_sprite();
        world.add_explosion(
            Entity::BulletExplosion(
                BulletExplosion::new(
                    alien_bounding_box.origin, 
                    bullet_explosion_sprite, 
                    world.get_bullet_explosion_time() as i32)));

        // remove alien from column index, needed to keep track of most left and right column
        // and update swarm speed... this is a bit of hack to emulate the feel of the original space invaders. seems
        // to feel about OK, but it is not an emulation of the orignal game, that relied on how the each alien were 
        // rendered (one per interrupt), thus speeding up naturally as more were killed!
        world.kill_alien(alien_index);
        //*world.get_mut_alien_swarm_speed() -= Duration::from_millis(9);

        // TODO: fixup the magic numbers below!
        if world.get_alien_dead() == World::number_aliens()-1 {
            *world.get_mut_alien_speed() += 10;
        }
        else if world.get_alien_dead() % 4 == 0 {
            //*world.get_mut_alien_swarm_speed() -= Duration::from_millis(9);
            *world.get_mut_alien_speed() += 3;
        }

        let mut updated_score = 0;
        if let Some(entity) = world.get_mut_entity(world.get_player()) {
            if let Entity::Player(player) = entity {
                player.bullet.bullet_mode = BulletMode::Fire;
                // add points to players score
                player.score += player_points_inc;
                updated_score = player.score;
            }
        }

        world.play_alien_explosion();

        // check high-score and update, if necessary
        if world.get_high_score() < updated_score as u32 {
            *world.get_mut_high_score() = updated_score as u32;
        }

        // should move to next level?
        if world.get_current_state() == GameState::Playing && world.get_alien_dead() == World::number_aliens() {
            world.set_current_state(GameState::NextLevel);
        }
    }
}