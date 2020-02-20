//! Description: 
//! 
//! 
//! Copyright © 2020 Benedict Gaster. All rights reserved.

use std::time::{Duration, Instant};
use either::*;
use rand::{RngCore};

use crate::sprite_sheet::{SpriteSheet, SheetJSON, AnimationJSON, Sprite};
use crate::entity::*;
use crate::animation::*;
use crate::math::*;
use crate::interface::*;
use crate::timer::*;

//------------------------------------------------------------------------------

type Time = Duration;

//------------------------------------------------------------------------------

const NUMBER_ALIEN_COLUMNS: usize = 11;
const NUMBER_ALIEN_ROWS: usize = 5;
const NUMBER_ALIENS: usize = NUMBER_ALIEN_COLUMNS * NUMBER_ALIEN_ROWS;

const PLAYER_BULLET_SPEED: i32 = 6;
const PLAYER_MOVEMENT: i32 = 4;

const BULLET_EXPLOSION_TIME: u64 = 24;

const PLAYER_DIED_DURATION: Time = Duration::from_millis(1000);

const ALIEN_INITIAL_SPEED: i32 = 2;
//const ALIEN_SWARM_INITIAL_SPEED: Time = 24;
const ALIEN_SWARM_INITIAL_SPEED: Time = Duration::from_millis(120);
const ALIEN_BULLET_START_DURATION: Time = Duration::from_millis(1000);
const ALIEN_BULLET_LESS_EIGHT_DURATION: Time = Duration::from_millis(70);
const ANIMATE_ALIEN_BULLET_DURATION: Time = Duration::from_millis(20);
const ALIEN_STEP_DOWN: u32 = 8;

const ALIEN_ONE_PADDING: u32 = 10;
const ALIEN_TOP_LEFT_X_START_POSITION: u32 = 220;
const ALIEN_TOP_LEFT_Y_START_POSITION: u32 = 80;
const ALIEN_SPACING_VERT: u32 = 35;
const ALIEN_SPACING_HORZ: u32 = 130;
const ALIEN_BULLET_INITIAL_SPEED: u32 = 6;

const PLAYER_TOP_LEFT_X_START_POSITION: u32 = 220;
const PLAYER_TOP_LEFT_Y_START_POSITION: u32 = 320;

const BOUNDING_BOX_TOP_LEFT_X: u32 = 10;
const BOUNDING_BOX_TOP_LEFT_Y: u32 = 10;
const BOUNDING_BOX_PADDING: i32 = 10;

const ALIEN_ENTITY_START: EntityIndex = 6;

pub const PLAYER_LIVES_TOP_LEFT_X_START_POSITION: u32 = 240;
pub const PLAYER_LIVES_TOP_LEFT_Y_START_POSITION: u32 = 370;

lazy_static! {
    static ref SCREEN_LINE: Rect = Rect::new(Point::new(0,362), Size::new(Interface::get_width(), 2));
}

//#[derive(Debug)]
pub struct World {
    // resources
    
    /// random number generator
    rng: Box<dyn RngCore>,

    /// sprite sheet for all sprites in game
    sprite_sheet: SpriteSheet,
    
    /// explosion when player bullet hits top of internal screen 
    player_bullet_explosion: Sprite,

    /// explosion when player is hit by alien bullet
    player_explosion: Animation,

    /// player died timer, used to delay when player was killed
    player_died_timer: Timer,

    /// has player died
    player_died: bool,
    

    /// explosion when alien bullet hits the ground 
    alien_bullet_explosion: Sprite,
    
    /// explosion when player bullet hits alien
    player_alien_bullet_explosion: Sprite,

    /// explosion when alien bullet hits player's bullet 
    alien_bullet_explosiion_with_player_bullet: Sprite,
    
    /// bounding box for playable area
    internal_rect: Rect,
    alien_swarm_direction: i32,

    /// current speed of alien swam
    alien_swam_speed: Time,
    alien_swarm_top_left_position: Point,
    alien_speed: i32,

    previous_time: Instant,
    lag: Duration,
    alien_dead: i32,
    
    // alien bullets
    next_alien_bullet_time: Timer,
    next_alien_bullet_type: AlienBulletType,
    animate_alien_bullet_time: Timer,

    // track number of live aliens in each of the columns
    // this is used to determine when alien swarm should move down a row, ie. when it has reached boundary
    alien_columns: [i32; NUMBER_ALIEN_COLUMNS],

    // entities

    entities: Vec<Option<Entity>>,
    player: EntityIndex,
    aliens: Vec<EntityIndex>,
    alien_bullet1: EntityIndex,
    alien_bullet2: EntityIndex,
    alien_bullet3: EntityIndex,
    explosions: Vec<EntityIndex>,
    ship:EntityIndex,
}

impl World {
    pub fn new(
        internal_rect: Rect,
        sprite_sheet: SpriteSheet, 
        player_bullet_explosion: Sprite,
        player_explosion: Animation,
        alien_bullet_explosion: Sprite,
        player_alien_bullet_explosion: Sprite,
        alien_bullet_explosiion_with_player_bullet: Sprite,
        alien_swarm_direction: i32,
        alien_swarm_top_left_position: Point,
        player: Entity, 
        alien_bullet1: Entity,
        alien_bullet2: Entity,
        alien_bullet3: Entity,
        alien_ens: Vec<Entity>, 
        ship: Entity) -> Self {

        let mut entities = vec![
            None, 
            Some(player), 
            Some(ship), 
            Some(alien_bullet1), Some(alien_bullet2), Some(alien_bullet3)];
        let player = 1;
        let ship   = 2;
        let alien_bullet1 = 3;
        let alien_bullet2 = 4;
        let alien_bullet3 = 5;

        let mut aliens = vec![];
        for (i, a) in alien_ens.iter().enumerate() {
            entities.push(Some((*a).clone()));
            aliens.push(i+ALIEN_ENTITY_START);
        }

        World {
            rng: Box::new(rand::thread_rng()),
            internal_rect,
            sprite_sheet,
            player_bullet_explosion,
            player_explosion,
            player_died_timer: Timer::new(PLAYER_DIED_DURATION),
            player_died: false,
            alien_bullet_explosion,
            player_alien_bullet_explosion,
            alien_bullet_explosiion_with_player_bullet,
            alien_swarm_direction,
            alien_swam_speed: ALIEN_SWARM_INITIAL_SPEED,
            alien_swarm_top_left_position,
            alien_speed: ALIEN_INITIAL_SPEED,
            //time: ,
            previous_time: Instant::now(),
            lag: Duration::new(0,0),
            alien_dead: 0,
            next_alien_bullet_time: Timer::new(ALIEN_BULLET_START_DURATION),
            next_alien_bullet_type: AlienBulletType::Plunger,
            animate_alien_bullet_time: Timer::new(ANIMATE_ALIEN_BULLET_DURATION),
            alien_columns: [NUMBER_ALIEN_ROWS as i32; NUMBER_ALIEN_COLUMNS],
            entities,
            player,
            aliens,
            alien_bullet1,
            alien_bullet2,
            alien_bullet3,
            explosions: vec![],
            ship,
        }
    }

    /// initial speed of alien bullets
    pub fn get_alien_bullet_initial_speed() -> u32 {
        ALIEN_BULLET_INITIAL_SPEED
    }

    /// generate a random column index [0,NUMBER_ALIEN_COLUMNS]
    pub fn gen_rand_column(&mut self) -> usize {
        self.rng.next_u64() as usize % NUMBER_ALIEN_COLUMNS
    }

    /// returns is the player is in process of dying boolean
    pub fn get_player_died(&self) -> bool {
        self.player_died
    }

    /// returns mutable ref for is the player is in process of dying
    pub fn get_mut_player_died(&mut self) -> &mut bool {
        &mut self.player_died
    }

    #[inline]
    pub fn has_player_died_timer_expired(&self) -> bool {
        self.player_died_timer.has_expired()
    }

    #[inline]
    pub fn reset_player_died__timer(&mut self) {
        self.player_died_timer.reset()
    }

    /// returns the ground rect for drawing and colision
    #[inline]
    pub fn get_ground() -> Rect {
        *SCREEN_LINE
    }

    /// returns number of pixels player moves left or right
    #[inline]
    pub fn player_movement() -> i32 {
        PLAYER_MOVEMENT
    }

    #[inline]
    pub fn player_start_position() -> Point {
        Point::new(PLAYER_TOP_LEFT_X_START_POSITION, PLAYER_TOP_LEFT_Y_START_POSITION)
    }

    /// returns elasped time since the last call
    #[inline]
    pub fn get_lag(&self) -> Duration {
        self.lag
    }

    #[inline]
    pub fn reset_lag(&mut self) {
        self.lag = Duration::new(0,0); 
    }

    /// drop killed alien from column index
    #[inline]
    pub fn kill_alien(&mut self, index: EntityIndex) {
        let offset = index % NUMBER_ALIEN_COLUMNS;
        self.alien_columns[offset] -= 1;

        // increment the global number of alien dead
        self.inc_alien_dead();
    }

    // returns right most column that contains a live alien 
    pub fn right_most_alien_column(&self) -> usize {
        let mut column = 0;
        for i in (0..NUMBER_ALIEN_COLUMNS).rev() {
            if self.alien_columns[i] != 0 {
                column = i;
                break;
            }
        }
        column
    }

    // returns the left most column that contains a live alien
    pub fn left_most_alien_column(&self) -> u32 {
        let mut column = 0;
        for i in 0..NUMBER_ALIEN_COLUMNS {
            if self.alien_columns[i] != 0 {
                column = i;
                break;
            }
        }
        column as u32
    }

    /// find the lowest alien, for a given column, that is alive
    pub fn lowest_alive_alien_in_column(&self, column: usize) -> Option<EntityIndex> {
        for row in (0..NUMBER_ALIEN_ROWS).rev() {
            let index = column + (row*NUMBER_ALIEN_COLUMNS);
            let entity_index = self.get_alien(index);
            if let Some(entity) = self.get_entity(entity_index) {
                if let Entity::Alien(alien) = entity {
                    if alien.is_alive {
                        return Some(entity_index);
                    }
                }
            }
        }
        None
    }

    #[inline]
    pub fn get_player_explosion_sprite(&self) -> Either<Sprite,Animation> {
        Right(self.player_explosion.clone())
    }

    #[inline]
    pub fn get_player_bullet_explosion_sprite(&self) -> Either<Sprite,Animation> {
        Left(self.player_bullet_explosion.clone())
    }

    #[inline]
    pub fn get_alien_bullet_explosion_sprite(&self) -> Either<Sprite,Animation> {
        Left(self.alien_bullet_explosion.clone())
    }

    #[inline]
    pub fn get_player_bullet_explosion(&self) -> Either<Sprite,Animation> {
        Left(self.player_bullet_explosion.clone())
    }

    #[inline]
    pub fn get_player_alien_bullet_explosion_sprite(&self) -> Either<Sprite,Animation> {
        Left(self.player_alien_bullet_explosion.clone())
    }

    #[inline]
    pub fn get_alien_bullet_explosiion_with_player_bullet(&self) -> Either<Sprite,Animation> {
        Left(self.alien_bullet_explosiion_with_player_bullet.clone())
    }

    /// returns the current speed of a players bullet (i.e. number of pixels it moves per animation)
    #[inline]
    pub fn get_player_bullet_speed(&self) -> i32 {
        PLAYER_BULLET_SPEED
    }

    /// returns the horz spacing between aliens
    #[inline]
    pub fn get_alien_spacing_horz(&self) -> u32 {
        ALIEN_SPACING_HORZ
    }

    /// returns the current speed of the alien swarm, which changes when there are fewer aliens
    #[inline]
    pub fn get_alien_swarm_speed(&self) -> Duration {
        self.alien_swam_speed
    }

    /// returns a mutable reference to the current speed of the alien swarm, which changes when there are fewer aliens
    #[inline]
    pub fn get_mut_alien_swarm_speed(&mut self) -> &mut Time {
        &mut self.alien_swam_speed
    }

    /// returns the number of pixels an alien moves down when the swarm reaches a boundary
    #[inline]
    pub fn get_alien_step_down(&self) -> u32 {
        ALIEN_STEP_DOWN
    }

    /// returns the amount of time a bullet explosion appears on the screen
    #[inline]
    pub fn get_bullet_explosion_time(&self) -> u64 {
        BULLET_EXPLOSION_TIME
    }
    
    /// retuns the number of columns in the alien swarm
    #[inline]
    pub fn get_number_alien_columns(&self) -> usize {
        NUMBER_ALIEN_COLUMNS
    }

    /// returns the current direction the alien swarm is moving
    #[inline]
    pub fn get_alien_swarm_direction(&self) -> i32 {
        self.alien_swarm_direction
    }

    /// returns a mutable reference to the current direction the alien swarm is moving
    #[inline]
    pub fn get_mut_alien_swarm_direction(&mut self) -> &mut i32 {
        &mut self.alien_swarm_direction
    }

    #[inline]
    pub fn get_alien_swarm_top_left_position(&self) -> Point {
        self.alien_swarm_top_left_position
    }

    #[inline]
    pub fn get_mut_alien_swarm_top_left_postion(&mut self) -> &mut Point {
        &mut self.alien_swarm_top_left_position
    }

    #[inline]
    pub fn get_alien_speed(&self) -> i32 {
        self.alien_speed
    }

    #[inline]
    pub fn get_mut_alien_speed(&mut self) -> &mut i32 {
        &mut self.alien_speed
    }

    /// return global time
    // TODO: use real time
    // #[inline]
    // pub fn get_time(&self) -> Time {
    //     self.time
    // }

    /// increment global
    // TODO: use real time
    //#[inline]
    // fn inc_time(&mut self) {
    //     self.time += 1;
    // }
    
    /// returns the number of aliens that have been killed
    #[inline]
    pub fn get_alien_dead(&self) -> i32 {
        self.alien_dead
    }

    /// increment the number of aliens that have been killed
    #[inline]
    fn inc_alien_dead(&mut self) {
        self.alien_dead += 1;
    }

    #[inline]
    pub fn reset_alien_dead(&mut self) {
        self.alien_dead = 0;
    }

    /// returns the total number of aliens
    #[inline]
    pub fn number_aliens() -> i32 {
        NUMBER_ALIENS as i32
    }

    /// returns the internal screen bounds, i.e. the bounding box of the playable area
    #[inline]
    pub fn get_bounds(&self) -> Rect {
        self.internal_rect
    }

    #[inline]
    pub fn get_alien_column(&self, index: usize) -> i32 {
        self.alien_columns[index]
    }

    #[inline]
    pub fn get_mut_alien_column(&mut self, index: usize) -> &mut i32 {
        &mut self.alien_columns[index]
    }

    /// returns the number of explosions that are currelty live
    #[inline]
    pub fn number_explosions(&self) -> usize {
        self.explosions.len()
    }

    /// iterator over live explosions
    #[inline]
    pub fn get_explosions<'a>(&'a self) -> impl Iterator<Item = EntityIndex> + 'a {
        self.explosions.iter().cloned()
    }

    /// returns an EntityIndex for an explosion, given an index
    #[inline]
    pub fn get_explosion_index(&self, index: usize) -> EntityIndex {
        self.explosions[index]
    }

    /// add a live explosion
    pub fn add_explosion(&mut self, explosion: Entity) {
        self.entities.push(Some(explosion));
        self.explosions.push(self.entities.len() - 1)
    }

    /// delete an explosion, thus it will no longer be live
    pub fn delete_explosion(&mut self, index: usize) {
        self.explosions.remove(index);
    }

    /// returns a ref to the main sprite sheet
    #[inline]
    pub fn get_sprite_sheet(&self) -> &SpriteSheet {
        &self.sprite_sheet
    }

    /// returns the entity index for the player
    // TODO: add a 2nd player
    #[inline]
    pub fn get_player(&self) -> EntityIndex {
        self.player
    }

    /// returns the entity index for the ship
    #[inline]
    pub fn get_ship(&self) -> EntityIndex {
        self.ship
    }

    /// returns an entity, which might not exist anymore so it is wrapped in Option
    #[inline]
    pub fn get_entity(&self, index: EntityIndex) -> Option<&Entity> {
        if index < self.entities.len() {
            if let Some(e) = &&self.entities[index] {
                return Some(e);
            }
        }
        None
    }

    #[inline]
    pub fn get_aliens<'a>(&'a self) -> impl Iterator<Item = EntityIndex> + 'a {
        self.aliens.iter().cloned()
    }

    #[inline]
    pub fn get_number_aliens(&self) -> usize {
        self.aliens.len()
    }

    #[inline]
    pub fn get_alien(&self, index: usize) -> EntityIndex {
        self.aliens[index]
    }

    #[inline]
    pub fn get_alien_rolling(&self) -> EntityIndex {
        self.alien_bullet1
    }

    #[inline]
    pub fn get_alien_plunger(&self) -> EntityIndex {
        self.alien_bullet2
    }

    #[inline]
    pub fn get_alien_squiggly(&self) -> EntityIndex {
        self.alien_bullet3
    }

    #[inline]
    pub fn get_alien_bullets(&self) -> [EntityIndex;3] {
        [self.alien_bullet1, self.alien_bullet2, self.alien_bullet3]
    }

    #[inline]
    pub fn get_next_alien_bullet_type(&self) -> AlienBulletType {
        self.next_alien_bullet_type
    }

    #[inline]
    pub fn inc_next_bullet_type(&mut self) {
        self.next_alien_bullet_type = self.next_alien_bullet_type.next();
    }

    #[inline]
    pub fn has_alien_bullet_timer_expired(&self) -> bool {
        self.next_alien_bullet_time.has_expired()
    }

    #[inline]
    pub fn reset_alien_bullet_timer(&mut self) {
        self.next_alien_bullet_time.reset()
    }

    #[inline]
    pub fn has_animate_alien_bullet_timer_expired(&self) -> bool {
        self.animate_alien_bullet_time.has_expired()
    }

    #[inline]
    pub fn reset_animate_alien_bullet_timer(&mut self) {
        self.animate_alien_bullet_time.reset()
    }

    #[inline]
    pub fn get_mut_entity(&mut self, index: EntityIndex) -> Option<&mut Entity> {
        if index < self.entities.len() {
            if let Some(e) = &mut self.entities[index] {
                return Some(e);
            }
        }
        None
    }

    // update the world, needs to be called each time around the main loop
    // to keep everything moving forward
    pub fn update(&mut self) {
        // update the time counter
        //self.inc_time();

        // handle any cleanup needed in the world and so on
        // nothing to do at the moment
        let now = Instant::now();
        let elasped_time = now - self.previous_time;
        self.previous_time = now;
        self.lag += elasped_time;
    }
}

// create the state of the inital game world
pub fn initial_world_state() -> World {

    //TODO: need to use relative paths for JSONs :-)

    // load JSON files for sprites and animations
    let sheet_json   = SheetJSON::new("/Users/br-gaster/dev/space-invaders/assets/sprite-sheet.json");
    let anis_json   = AnimationJSON::from_json("/Users/br-gaster/dev/space-invaders/assets/sprite-animation.json");

    // create resources

    // sprite sheet used by render and indexed by sprites (and by implication animations)
    let sprite_sheet = SpriteSheet::new("/Users/br-gaster/dev/space-invaders/assets/sprite-sheet.png");

    // TODO: fix to be below text, once we have text
    let bounds = Rect::new(
        Point::new(BOUNDING_BOX_TOP_LEFT_X*4, BOUNDING_BOX_TOP_LEFT_Y*4), 
        Size::new(
            (Interface::get_width() as i32 - BOUNDING_BOX_PADDING) as u32 * 4,
            (Interface::get_height() as i32 - BOUNDING_BOX_PADDING) as u32 * 4));

    let alien_swarm_position = Point::new(ALIEN_TOP_LEFT_X_START_POSITION, ALIEN_TOP_LEFT_Y_START_POSITION);
    let alien_swarm_direction = 1;

    // TODO: add clock

    // create entities
    
    let mut aliens = vec![];
    let transform = Vector::new(ALIEN_SPACING_HORZ,0);

    // add single row of alien 1
    let alien = Animation::new(anis_json.get(&"Alien1".to_string()).unwrap(), &sheet_json);
    let mut pos = Point::new(ALIEN_TOP_LEFT_X_START_POSITION+ALIEN_ONE_PADDING, ALIEN_TOP_LEFT_Y_START_POSITION);
    let bounding_box = alien.get_bounding_box();
    for _ in 0..11 {
        aliens.push(Entity::Alien(Alien::new(pos, 30, bounding_box, alien.clone())));
        pos += transform;
    }

    // add 1st row of alien 2
    let alien = Animation::new(anis_json.get(&"Alien2".to_string()).unwrap(), &sheet_json);
    let mut pos = Point::new(ALIEN_TOP_LEFT_X_START_POSITION, ALIEN_TOP_LEFT_Y_START_POSITION + ALIEN_SPACING_VERT);
    let bounding_box = alien.get_bounding_box();
    for _ in 0..11 {
        aliens.push(Entity::Alien(Alien::new(pos, 20, bounding_box, alien.clone())));
        pos += transform;
    }

    // add 2nd row of alien 2
    let alien = Animation::new(anis_json.get(&"Alien2".to_string()).unwrap(), &sheet_json);
    let mut pos = Point::new(ALIEN_TOP_LEFT_X_START_POSITION, ALIEN_TOP_LEFT_Y_START_POSITION + ALIEN_SPACING_VERT*2);
    let bounding_box = alien.get_bounding_box();
    for _ in 0..11 {
        aliens.push(Entity::Alien(Alien::new(pos, 20, bounding_box, alien.clone())));
        pos += transform;
    }

    // add 1st row of alien 3
    let alien = Animation::new(anis_json.get(&"Alien3".to_string()).unwrap(), &sheet_json);
    let mut pos = Point::new(ALIEN_TOP_LEFT_X_START_POSITION, ALIEN_TOP_LEFT_Y_START_POSITION + ALIEN_SPACING_VERT*3);
    let bounding_box = alien.get_bounding_box();
    for _ in 0..11 {
        aliens.push(Entity::Alien(Alien::new(pos, 10, bounding_box, alien.clone())));
        pos += transform;
    }

    // add 2nd row of alien 3
    let alien = Animation::new(anis_json.get(&"Alien3".to_string()).unwrap(), &sheet_json);
    let mut pos = Point::new(ALIEN_TOP_LEFT_X_START_POSITION, ALIEN_TOP_LEFT_Y_START_POSITION + ALIEN_SPACING_VERT*4);
    let bounding_box = alien.get_bounding_box();
    for _ in 0..11 {
        aliens.push(Entity::Alien(Alien::new(pos, 10, bounding_box, alien.clone())));
        pos += transform;
    }

    let alien_bullet1_ani = Animation::new(anis_json.get(&"AlienBullet1".to_string()).unwrap(), &sheet_json);
    let alien_bullet1_bounding_box = alien_bullet1_ani.get_bounding_box();
    let alien_bullet1 = Entity::Bullet(
        Bullet::new(Point::new(0,0), Right(alien_bullet1_ani), alien_bullet1_bounding_box));
    let s = sheet_json.frames.get("alien_bullet2.png").unwrap();
    let alien_bullet2 = Entity::Bullet(Bullet::new(
        Point::new(0,0), 
        Left(Sprite::new(
            s.frame.x as u32, 
            s.frame.y as u32, 
            s.frame.w as u32, 
            s.frame.h as u32)),
        Rect::new(Point::new(0,0), Size::new(s.frame.w as u32, s.frame.h as u32))));
    let alien_bullet3_ani = Animation::new(anis_json.get(&"AlienBullet3".to_string()).unwrap(), &sheet_json);
    let alien_bullet3_bounding_box = alien_bullet3_ani.get_bounding_box();
    let alien_bullet3 = Entity::Bullet(
        Bullet::new(Point::new(0,0), Right(alien_bullet3_ani), alien_bullet3_bounding_box));

    // player 
    let s = sheet_json.frames.get("Player.png").unwrap();
    //let s = sheet_json.frames.get("Player.png").unwrap();
    let player_sprite = Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32);
    let bounding_box = Rect::new(Point::new(0,0), Size::new(s.frame.w as u32, s.frame.h as u32));
    let bullet_sprite = sheet_json.frames.get("player_bullet.png").unwrap();
    let player_bullet = Bullet::new(
        Point::new(0,0), 
        Left(Sprite::new(
            bullet_sprite.frame.x as u32, 
            bullet_sprite.frame.y as u32, 
            bullet_sprite.frame.w as u32, 
            bullet_sprite.frame.h as u32)),
        Rect::new(Point::new(0,0), Size::new(bullet_sprite.frame.w as u32, bullet_sprite.frame.h as u32)));
    let player = Entity::Player(
        Player::new(
            Point::new(PLAYER_TOP_LEFT_X_START_POSITION, PLAYER_TOP_LEFT_Y_START_POSITION), 
            player_sprite,
            player_bullet,
            bounding_box));

    let explosion_sprite = sheet_json.frames.get("bullet_top_bang.png").unwrap();
    //let explosion_sprite = sheet_json.frames.get("block.png").unwrap();
    let player_bullet_explosion_sprite = Sprite::new(
        explosion_sprite.frame.x as u32, 
        explosion_sprite.frame.y as u32,
        explosion_sprite.frame.w as u32, 
        explosion_sprite.frame.h as u32);

    let explosion_sprite = sheet_json.frames.get("alien_bullet_explosiion_with_player_bullet.png").unwrap();
    let alien_bullet_explosiion_with_player_bullet_sprite = Sprite::new(
        explosion_sprite.frame.x as u32, 
        explosion_sprite.frame.y as u32,
        explosion_sprite.frame.w as u32, 
        explosion_sprite.frame.h as u32);

    let player_explosion_sprite = Animation::new(anis_json.get(&"PlayerExplosion".to_string()).unwrap(), &sheet_json);
    // let explosion_sprite = sheet_json.frames.get("player_explosion.png").unwrap();
    // //let explosion_sprite = sheet_json.frames.get("block.png").unwrap();
    // let player_explosion_sprite = Sprite::new(
    //     explosion_sprite.frame.x as u32, 
    //     explosion_sprite.frame.y as u32,
    //     explosion_sprite.frame.w as u32, 
    //     explosion_sprite.frame.h as u32);

    let explosion_sprite = sheet_json.frames.get("alien_bullet_explosiion.png").unwrap();
    let alien_bullet_explosion_sprite = Sprite::new(
        explosion_sprite.frame.x as u32, 
        explosion_sprite.frame.y as u32,
        explosion_sprite.frame.w as u32, 
        explosion_sprite.frame.h as u32);

    let alien_explosion_sprite = sheet_json.frames.get("alien_explosion.png").unwrap();
    //let explosion_sprite = sheet_json.frames.get("block.png").unwrap();
    let player_alien_bullet_explosion_sprite = Sprite::new(
        alien_explosion_sprite.frame.x as u32, 
        alien_explosion_sprite.frame.y as u32,
        alien_explosion_sprite.frame.w as u32, 
        alien_explosion_sprite.frame.h as u32);

    // ship
    let s = sheet_json.frames.get("ship.png").unwrap();
    let ship_sprite = Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32);
    let ship = Entity::Ship(Ship::new(Point::new(10, 200), ship_sprite));

    World::new(
        bounds, 
        sprite_sheet,
        player_bullet_explosion_sprite,
        player_explosion_sprite,
        alien_bullet_explosion_sprite,
        player_alien_bullet_explosion_sprite,
        alien_bullet_explosiion_with_player_bullet_sprite,
        alien_swarm_direction, 
        alien_swarm_position, 
        player,  
        alien_bullet1,
        alien_bullet2,
        alien_bullet3,
        aliens,
        ship)
}