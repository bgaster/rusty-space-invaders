//! Description: 
//! 
//! Handle collisions, which for space invaders means handle bullets hitting things :-)
//! 
//! Copyright © 2020 Benedict Gaster. All rights reserved.
//! 

use crate::entity::*;
use crate::world::*;
use crate::math::*;

fn collides_with_barrier(bounding_box: &Rect, barriers_info: &Vec<(EntityIndex, Rect)>) -> bool {
    for i in barriers_info {
        if i.1.intersects(bounding_box) {
            return true;
        }
    }

    false
}

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
                    Size::new(player_bullet_bounding_box.size.width*4, player_bullet_bounding_box.size.height);
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
                         Size::new(barrier.bounding_box.size.width*4, barrier.bounding_box.size.height))));
            }
        }
    }

    let mut player_killed = false;
    let mut player_bullet_killed = false;
    let mut explosions: Vec<BulletExplosion> = vec![];
    let bullet_explosion_sprite = world.get_alien_bullet_explosion_sprite();
    // handle alien bullet collisions
    for index in world.get_alien_bullets().iter() {
        if let Some(entity) = world.get_mut_entity(*index) {
            if let Entity::Bullet(bullet) = entity {
                if bullet.bullet_mode == BulletMode::InFlight {
                    
                    // check for colision with player
                    let mut bullet_bounding_box = bullet.bounding_box;
                    bullet_bounding_box.origin = bullet.position;

                    // first check if collides with barrier
                    if collides_with_barrier(&bullet_bounding_box, &barriers) {
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
                if collides_with_barrier(&player_bullet_bounding_box, &barriers) {
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

        if let Some(entity) = world.get_mut_entity(world.get_player()) {
            if let Entity::Player(player) = entity {
                player.bullet.bullet_mode = BulletMode::Fire;
                // add points to players score
                player.score += player_points_inc;
            }
        }

        world.play_alien_explosion();
    }
}