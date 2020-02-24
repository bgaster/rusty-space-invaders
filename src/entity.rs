//! Description: 
//! 
//! Simple entity system for our game. We could of course use an ECS, but as
//! the whole point is to port this game over to the 32blit when it's ready,
//! this seemed like overkill and potentially impact on portability. I want
//! an engine that is very simply and low overhead.
//! 
//! Copyright © 2020 Benedict Gaster. All rights reserved.
//! 

use either::*;

use crate::sprite_sheet::{Sprite, SpriteMask};
use crate::animation::*;
use crate::math::*;

pub type EntityIndex = usize;

const PLAYER_START_LIVES: i32 = 3;
const PLAYER_INITIAL_SCORE: i32 = 0;

#[derive(Debug, Clone, PartialEq)]
pub enum BulletMode {
    Fire,
    InFlight,
    Explode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlienBulletType {
    Rolling = 0,
    Plunger = 1,
    Squiggly = 2,
}

impl AlienBulletType {
    pub fn next(&self) -> Self {
        let n = *self as u32 + 1;
        match n % 3 {
            0 => AlienBulletType::Rolling,
            1 => AlienBulletType::Plunger,
            2 => AlienBulletType::Squiggly,
            _ => AlienBulletType::Rolling, // hmm we should not need this, but compiler can't detect...
        }
    }
}

/// Representation of bullet entity
#[derive(Debug, Clone)]
pub struct Bullet {
    /// screen position of bullet
    pub position: Point,
    /// some bullets have only a single sprite, some are animated by many
    pub sprite: Either<Sprite,Animation>,
    /// current status of bullet, e.g. is in in flight
    pub bullet_mode: BulletMode,
    /// axis alined bounding box
    pub bounding_box: Rect,
}

impl Bullet {
    pub fn new(position: Point, sprite: Either<Sprite,Animation>, bounding_box: Rect,) -> Self {
        Bullet {
            position,
            sprite,
            bullet_mode: BulletMode::Fire,
            bounding_box,
        }
    }

    pub fn get_bounding_box(&self) -> Rect {
        Rect::new(
            self.position,
            Size::new(self.bounding_box.size.width*4,self.bounding_box.size.height))
    }
}

/// Representation of bullet explosion entity
#[derive(Debug, Clone)]
pub struct BulletExplosion {
    pub position: Point,
    pub sprite: Either<Sprite,Animation>,
    pub framecount: i32,
}

impl BulletExplosion {
    pub fn new(position: Point, sprite: Either<Sprite,Animation>, framecount: i32) -> Self {
        BulletExplosion {
            position,
            sprite,
            framecount,
        }
    }
}


/// Representation of player entity
#[derive(Debug, Clone)]
pub struct Player {
    /// screen position of barrier
    pub position: Point,
    /// player sprite
    pub sprite: Sprite,
    /// bullet entity for player, as they can only ever be one we keep it here
    pub bullet: Bullet,
    /// axis alined bounding box
    pub bounding_box: Rect,
    /// number of lives remaining
    pub lives_remaining: i32,
    /// current player's score
    pub score: i32,
}

impl Player {
    pub fn new(position: Point, sprite: Sprite, bullet: Bullet, bounding_box: Rect) -> Self {
        Player {
            position,
            sprite,
            bullet,
            bounding_box,
            lives_remaining: PLAYER_START_LIVES,
            score: PLAYER_INITIAL_SCORE,
        }
    }

    pub fn get_bounding_box(&self) -> Rect {
        // TODO: resolve the *4 hack!!
        Rect::new(
            self.position,
            Size::new(self.bounding_box.size.width*4,self.bounding_box.size.height))
    }
}

#[derive(Debug, Clone)]
pub struct Barrier {
    /// screen position of barrier 
    pub position: Point,
    /// barrier sprite
    pub sprite: Sprite,
    /// mask used for colisions with sprite... when the barrier is hit the mask is update to represent the explosion
    pub mask: SpriteMask,
    /// axis alined bounding box
    pub bounding_box: Rect,
}

impl Barrier {
    /// create an barrier
    pub fn new(position: Point, sprite: Sprite, mask: SpriteMask, bounding_box: Rect,) -> Self {
        Barrier {
            position,
            sprite,
            mask,
            bounding_box,
        }
    }

    /// barriers bounding box
    pub fn get_bounding_box(&self) -> Rect {
        Rect::new(
            self.position,
            Size::new(self.bounding_box.size.width*4,self.bounding_box.size.height))
    }
}


#[derive(Debug, Clone)]
pub struct Alien {
    pub position: Point,
    pub points: i32,
    pub bounding_box: Rect,
    pub animation: Animation,
    pub is_alive: bool,
}

impl Alien {
    pub fn new(
        position: Point, 
        points: i32,
        bounding_box: Rect,
        animation: Animation) -> Self {
        Alien {
            position,
            points,
            bounding_box,
            animation,
            is_alive: true,
        }
    }

    pub fn get_bounding_box(&self) -> Rect {
        // TODO: resolve the *4 hack!!
        Rect::new(
            self.position,
            Size::new(self.bounding_box.size.width*4,self.bounding_box.size.height))
    }
}

#[derive(Debug, Clone)]
pub struct Ship {
    pub position: Point,
    pub sprite: Sprite,
    pub points: i32,
    pub is_alive: bool,
}

impl Ship {
    pub fn new(       
        position: Point, 
        sprite: Sprite) -> Self {
        Ship {
            position,
            sprite,
            points: 0, // this randomly generated at generation time
            is_alive: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Entity {
    Player(Player),
    Alien(Alien),
    Ship(Ship),
    Bullet(Bullet),
    BulletExplosion(BulletExplosion),
    Barrier(Barrier),
}