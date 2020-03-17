//! Description: 
//! 
//! The world... probably not the best design, as it has grown larger than I 
//! might have liked :-)
//! 
//! Copyright © 2020 Benedict Gaster. All rights reserved.

use std::time::{Duration, Instant};
use either::*;
use rand::{RngCore};

use crate::sprite_sheet::{SpriteSheet, SheetJSON, AnimationJSON, Sprite, SpriteMask};
use crate::entity::*;
use crate::animation::*;
use crate::math::*;
use crate::interface::*;
use crate::timer::*;
use crate::text::*;
use crate::sound::*;
use crate::config::*;

//------------------------------------------------------------------------------

type Time = Duration;

//------------------------------------------------------------------------------
// Constants used throughout the game, generally only accessable via world
//------------------------------------------------------------------------------


const NUMBER_ALIEN_COLUMNS: usize = 11;
const NUMBER_ALIEN_ROWS: usize = 5;
const NUMBER_ALIENS: usize = NUMBER_ALIEN_COLUMNS * NUMBER_ALIEN_ROWS;

const PLAYER_BULLET_SPEED: i32 = 6;
const PLAYER_MOVEMENT: i32 = 4;

const BULLET_EXPLOSION_TIME: u64 = 24;

const PLAYER_DIED_DURATION: Time = Duration::from_millis(1000);

const ALIEN_INITIAL_SPEED: i32 = 2;
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
const PLAYER_TOP_LEFT_Y_START_POSITION: u32 = 360;

const BARRIER_TOP_LEFT_X_START_POSITION: u32 = 270;
const BARRIER_TOP_LEFT_Y_START_POSITION: u32  = 310;
const BARRIER_SPACING_HORZ: u32 = 340;

const BOUNDING_BOX_TOP_LEFT_X: u32 = 10;
const BOUNDING_BOX_TOP_LEFT_Y: u32 = 10;
const BOUNDING_BOX_PADDING: i32 = 10;

pub const PLAYER_LIVES_TOP_LEFT_X_START_POSITION: u32 = 240;
pub const PLAYER_LIVES_TOP_LEFT_Y_START_POSITION: u32 = 410;

lazy_static! {
    static ref SCREEN_LINE: Rect = Rect::new(Point::new(0,400), Size::new(Interface::get_width(), 2));
}

/// Current state of game
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    /// actively being played
    Playing,
    /// inbetween levels
    NextLevel,
    /// gameplay is paused 
    Paused,
    /// displaying splash screen
    Splash,
    /// player has lost all lives and inbetween new game state
    GameOver,
}

//#[derive(Debug)]
pub struct World {

    /// current state of the game
    current_state: GameState,

    // resources
    
    /// random number generator
    rng: Box<dyn RngCore>,

    /// sounds
    sound: Sound,

    /// current music bpm index
    current_bpm: usize,

    /// sprite sheet for all sprites in game
    sprite_sheet: SpriteSheet,
    
    splash: Sprite,

    /// text stuff
    digits: Digits,

    /// score text stuff
    score_text: Score,

    /// current level,
    current_level: u32,

    /// current high score
    high_score: u32,

    /// explosion when player bullet hits top of internal screen 
    player_bullet_explosion: Sprite,

    /// explosion when player is hit by alien bullet
    player_explosion: Animation,

    /// player died timer, used to delay when player was killed
    player_died_timer: Timer,

    /// has player died
    player_died: bool,

    /// shield bullet explosion mask
    shield_bullet_explosion_mask: SpriteMask,

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

    /// track number of live aliens in each of the columns
    /// this is used to determine when alien swarm should move down a row, ie. when it has reached boundary
    alien_columns: [i32; NUMBER_ALIEN_COLUMNS],

    // entities

    entities: Vec<Option<Entity>>,
    player: EntityIndex,
    barriers: Vec<EntityIndex>,
    aliens: Vec<EntityIndex>,
    alien_bullet1: EntityIndex,
    alien_bullet2: EntityIndex,
    alien_bullet3: EntityIndex,
    /// temporary list of explosions, when bullets hit things
    explosions: Vec<EntityIndex>,
    ship:EntityIndex,
}

impl World {
    /// creates a world
    /// 
    /// # Arguments
    pub fn new(
        internal_rect: Rect,
        sound: Sound,
        sprite_sheet: SpriteSheet, 
        splash: Sprite,
        digits: Digits,
        score_text: Score,
        high_score: u32,
        shield_bullet_explosion_mask: SpriteMask,
        player_bullet_explosion: Sprite,
        player_explosion: Animation,
        alien_bullet_explosion: Sprite,
        player_alien_bullet_explosion: Sprite,
        alien_bullet_explosiion_with_player_bullet: Sprite,
        alien_swarm_direction: i32,
        alien_swarm_top_left_position: Point,
        player: Entity, 
        barriers_ens: Vec<Entity>,
        alien_bullet1: Entity,
        alien_bullet2: Entity,
        alien_bullet3: Entity,
        alien_ens: Vec<Entity>, 
        ship: Entity) -> Self {

        // add player, UFO, and alien bullet entities 
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

        // add barrier entities
        let mut barriers = vec![];
        for b in barriers_ens.iter() {
            entities.push(Some((*b).clone()));
            barriers.push(entities.len()-1);
        }

        // add alien entities
        let mut aliens = vec![];
        for a in alien_ens.iter() {
            entities.push(Some((*a).clone()));
            aliens.push(entities.len()-1);
        }

        // create the world
        World {
            current_state: GameState::Splash,
            rng: Box::new(rand::thread_rng()),
            internal_rect,
            sound,
            current_bpm: 0,
            sprite_sheet,
            splash,
            digits,
            score_text,
            high_score,
            current_level: 1,
            player_bullet_explosion,
            player_explosion,
            player_died_timer: Timer::new(PLAYER_DIED_DURATION),
            player_died: false,
            shield_bullet_explosion_mask,
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
            barriers,
            aliens,
            alien_bullet1,
            alien_bullet2,
            alien_bullet3,
            explosions: vec![],
            ship,
        }
    }

    #[inline]
    pub fn get_current_bpm(&self) -> usize {
        self.current_bpm
    }

    #[inline]
    pub fn set_current_bpm(&mut self, bpm: usize)  {
        self.current_bpm = bpm;
    }

    #[inline]
    pub fn get_high_score(&self) -> u32 {
        self.high_score
    }

    #[inline]
    pub fn get_mut_high_score(&mut self) -> &mut u32 {
        &mut self.high_score
    }

    #[inline]
    pub fn get_current_level(&self) -> u32 {
        self.current_level
    }

    #[inline]
    pub fn get_mut_current_level(&mut self) -> &mut u32 {
        &mut self.current_level
    }

    #[inline]
    pub fn current_level_inc(&mut self) {
        self.current_level += 1;
    }

    #[inline]
    pub fn get_shield_bullet_explosion_mask(&self) -> SpriteMask {
        self.shield_bullet_explosion_mask.clone()
    }

    /// play sound track at a particular speed
    #[inline]
    pub fn play_music(&mut self, bpm: usize) {
        self.sound.play_music(bpm);
    }

    /// pause sound track
    #[inline]
    pub fn pause_music(&self) {
        self.sound.pause_music();
    }

    /// play sound effect for alien explosion
    #[inline]
    pub fn play_alien_explosion(&self) {
        self.sound.play_alien_explosion();
    }

    #[inline]
    pub fn play_player_explosion(&self) {
        self.sound.play_player_explosion();
    }

    #[inline]
    pub fn get_sound(&self) -> &Sound {
        &self.sound
    }

    #[inline]
    pub fn play_player_shot(&self) {
        self.sound.play_player_shot();
    }

    /// return the current playing state the game is in
    #[inline]
    pub fn get_current_state(&self) -> GameState {
        self.current_state
    }

    /// set the current playing state
    #[inline]
    pub fn set_current_state(&mut self, state: GameState) {
        self.current_state = state;
    }

    /// digits for drawing numbers
    #[inline]
    pub fn get_digits(&self) -> &Digits {
        &self.digits
    }

    /// score text for drawing
    #[inline]
    pub fn get_score_text(&self) -> &Score {
        &self.score_text
    }

    /// initial speed of alien bullets
    #[inline]
    pub fn get_alien_bullet_initial_speed() -> u32 {
        ALIEN_BULLET_INITIAL_SPEED
    }

    /// generate a random column index [0,NUMBER_ALIEN_COLUMNS]
    #[inline]
    pub fn gen_rand_column(&mut self) -> usize {
        self.rng.next_u64() as usize % NUMBER_ALIEN_COLUMNS
    }

    /// returns is the player is in process of dying boolean
    #[inline]
    pub fn get_player_died(&self) -> bool {
        self.player_died
    }

    /// returns mutable ref for is the player is in process of dying
    #[inline]
    pub fn get_mut_player_died(&mut self) -> &mut bool {
        &mut self.player_died
    }

    #[inline]
    pub fn has_player_died_timer_expired(&self) -> bool {
        self.player_died_timer.has_expired()
    }

    #[inline]
    pub fn reset_player_died_timer(&mut self) {
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
    pub fn get_splash_screen_sprite(&self) -> Sprite {
        self.splash.clone()
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
    pub fn get_barriers<'a>(&'a self) -> impl Iterator<Item = EntityIndex> + 'a {
        self.barriers.iter().cloned()
    }

    #[inline]
    pub fn get_number_barriers(&self) -> usize {
        self.barriers.len()
    }

    #[inline]
    pub fn get_barrier(&self, index: usize) -> EntityIndex {
        self.barriers[index]
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

/// create the state of the inital game world
///
/// Arguments
/// 
/// * `config` - Game configuration file, contains high score and so on
pub fn initial_world_state(config: &Config) -> World {

    // load JSON files for sprites and animations
    //let sheet_json   = SheetJSON::new("/Users/br-gaster/dev/space-invaders/assets/sprite-sheet.json");
    let sheet_json   = SheetJSON::new("./assets/sprite-sheet.json");
    let anis_json   = AnimationJSON::from_json("./assets/sprite-animation.json");

    // create resources

    // sprite sheet used by render and indexed by sprites (and by implication animations)
    let sprite_sheet = SpriteSheet::new("./assets/sprite-sheet.png");

    // TODO: fix to be below text, once we have text
    let bounds = Rect::new(
        Point::new(BOUNDING_BOX_TOP_LEFT_X*4, BOUNDING_BOX_TOP_LEFT_Y*4), 
        Size::new(
            (Interface::get_width() as i32 - BOUNDING_BOX_PADDING) as u32 * 4,
            (Interface::get_height() as i32 - BOUNDING_BOX_PADDING) as u32 * 4));

    let alien_swarm_position = Point::new(ALIEN_TOP_LEFT_X_START_POSITION, ALIEN_TOP_LEFT_Y_START_POSITION);
    let alien_swarm_direction = 1;

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
    let bullet_sprite = sheet_json.frames.get("player_bullet_small.png").unwrap();
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

    // barriers
    let s = sheet_json.frames.get("barrier.png").unwrap();
    let barrier_sprite = Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32);
    let barrier_mask   = barrier_sprite.create_mask(&sprite_sheet);

    let barriers = vec![
        Entity::Barrier(Barrier::new(
            Point::new(BARRIER_TOP_LEFT_X_START_POSITION,BARRIER_TOP_LEFT_Y_START_POSITION), 
            barrier_sprite.clone(), 
            barrier_mask.clone(), 
            Rect::new(
                Point::new(BARRIER_TOP_LEFT_X_START_POSITION,BARRIER_TOP_LEFT_Y_START_POSITION), 
                Size::new(s.frame.w as u32, s.frame.h as u32)))),
        Entity::Barrier(Barrier::new(
            Point::new(
                BARRIER_TOP_LEFT_X_START_POSITION + bounding_box.size.width + BARRIER_SPACING_HORZ,
                BARRIER_TOP_LEFT_Y_START_POSITION), 
            barrier_sprite.clone(), 
            barrier_mask.clone(),
            Rect::new(
                Point::new(
                    BARRIER_TOP_LEFT_X_START_POSITION + bounding_box.size.width + BARRIER_SPACING_HORZ,
                    BARRIER_TOP_LEFT_Y_START_POSITION), 
                Size::new(s.frame.w as u32, s.frame.h as u32)))),
        Entity::Barrier(Barrier::new(
            Point::new(
                BARRIER_TOP_LEFT_X_START_POSITION + (bounding_box.size.width + BARRIER_SPACING_HORZ)*2,
                BARRIER_TOP_LEFT_Y_START_POSITION), 
            barrier_sprite.clone(), 
            barrier_mask.clone(),
            Rect::new(
                Point::new(
                    BARRIER_TOP_LEFT_X_START_POSITION + (bounding_box.size.width + BARRIER_SPACING_HORZ)*2,
                    BARRIER_TOP_LEFT_Y_START_POSITION), 
                Size::new(s.frame.w as u32, s.frame.h as u32)))),
        Entity::Barrier(Barrier::new(
            Point::new(
                BARRIER_TOP_LEFT_X_START_POSITION + (bounding_box.size.width + BARRIER_SPACING_HORZ)*3,
                BARRIER_TOP_LEFT_Y_START_POSITION), 
            barrier_sprite, 
            barrier_mask,
            Rect::new(
                Point::new(
                    BARRIER_TOP_LEFT_X_START_POSITION + (bounding_box.size.width + BARRIER_SPACING_HORZ)*3,
                    BARRIER_TOP_LEFT_Y_START_POSITION), 
                Size::new(s.frame.w as u32, s.frame.h as u32))))];

    let explosion_sprite = sheet_json.frames.get("alien_bullet_explosiion_with_player_bullet.png").unwrap();
    let alien_bullet_explosiion_with_player_bullet_sprite = Sprite::new(
        explosion_sprite.frame.x as u32, 
        explosion_sprite.frame.y as u32,
        explosion_sprite.frame.w as u32, 
        explosion_sprite.frame.h as u32);

    let player_explosion_sprite = Animation::new(anis_json.get(&"PlayerExplosion".to_string()).unwrap(), &sheet_json);

    let explosion_sprite = sheet_json.frames.get("alien_bullet_explosiion.png").unwrap();
    let alien_bullet_explosion_sprite = Sprite::new(
        explosion_sprite.frame.x as u32, 
        explosion_sprite.frame.y as u32,
        explosion_sprite.frame.w as u32, 
        explosion_sprite.frame.h as u32);

    
    // bullet explosion mask, used to cut out holes from sheild when it bullet collides
    let barrier_explosion_sprite_mask = sheet_json.frames.get("bullet_barrier_mask.png").unwrap();
    let barrier_explosion_sprite_mask = Sprite::new(
        barrier_explosion_sprite_mask.frame.x as u32, 
        barrier_explosion_sprite_mask.frame.y as u32,
        barrier_explosion_sprite_mask.frame.w as u32, 
        barrier_explosion_sprite_mask.frame.h as u32);
    let shield_bullet_explosion_mask = barrier_explosion_sprite_mask.create_mask(&sprite_sheet);

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

    let s = sheet_json.frames.get("splash.png").unwrap();
    let splash_sprite = Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32);

    // load sounds
    let sound = Sound::new(
        "./assets/sounds/player_shoot_16bit.wav",
        "./assets/sounds/player_explosion_16bit.wav",
        "./assets/sounds/alien_explosion_16bit.wav",
        vec!["./assets/sounds/invader_march_80bpm.wav",
            "./assets/sounds/invader_march_100bpm.wav",
            "./assets/sounds/invader_march_120bpm.wav",
            "./assets/sounds/invader_march_140bpm.wav",
            "./assets/sounds/invader_march_160bpm.wav",
            "./assets/sounds/invader_march_180bpm.wav",
            "./assets/sounds/invader_march_200bpm.wav"]);

    // load text
    let digits = Digits::new(&sheet_json);

    // load text
    let score_text = Score::new(&sheet_json);

    // finally, create the world
    World::new(
        bounds, 
        sound,
        sprite_sheet,
        splash_sprite,
        digits,
        score_text,
        config.get_high_score(),
        shield_bullet_explosion_mask,
        player_bullet_explosion_sprite,
        player_explosion_sprite,
        alien_bullet_explosion_sprite,
        player_alien_bullet_explosion_sprite,
        alien_bullet_explosiion_with_player_bullet_sprite,
        alien_swarm_direction, 
        alien_swarm_position, 
        player,  
        barriers,
        alien_bullet1,
        alien_bullet2,
        alien_bullet3,
        aliens,
        ship)
}

/// reset set the player to beginning of round state
/// 
/// # Arguments
/// 
/// * `world` Game world to be updated
fn reset_barriers(world: &mut World) {
    
    // get ref to sprite sheet 
    let sheet = world.get_sprite_sheet();

    // get list if masks, one for each barrier
    let mut masks = vec![];
    for index in world.get_barriers() {
        if let Some(entity) = world.get_entity(index) {
            if let Entity::Barrier(barrier) = entity {
                masks.push(barrier.sprite.create_mask(&sheet));
            }
        }
    }

    // now update the actual barriers with their initial mask, created above
    for index in 0..world.get_number_barriers() {
        if let Some(entity) = world.get_mut_entity(world.get_barrier(index)) {
            if let Entity::Barrier(barrier) = entity {
                barrier.mask = masks[index].clone();
            }
        }
    }
}

/// reset set the player to beginning of round state
/// 
/// # Arguments
/// 
/// * `world` Game world to be updated
fn reset_player(reset_lives_score: bool, world: &mut World) {

    // need to make sure this is reset
    world.player_died = false;

    if let Some(entity) = world.get_mut_entity(world.get_player()) {
        if let Entity::Player(player) = entity {
            player.position = Point::new(PLAYER_TOP_LEFT_X_START_POSITION, PLAYER_TOP_LEFT_Y_START_POSITION);
            player.bounding_box.origin = player.position;
            if reset_lives_score {
                player.score = PLAYER_INITIAL_SCORE;
                player.lives_remaining = PLAYER_START_LIVES;
            }
        }
    }
}

/// clear any outstanding explosions
/// 
/// # Arguments
/// 
/// * `world` - Game world to be updated
pub fn reset_explosions(world: &mut World) {
    world.explosions.clear();
}

/// reset set the alien to beginning of round state, their y position varies depending on the round
/// 
/// # Arguments
/// 
/// * `world` Game world to be updated
pub fn reset_aliens(round: u32, world: &mut World) {

    // The first round is set in initial world, then
    //  - for round 2 aliens start lower
    //  - for round 3,4,5 start lower again
    //  - for all other rounds they start lower again, but do not get any lower
    let vert_offset = match round {
        1 => 0,
        2 => 5*4,
        3 | 4 | 5 => 10*4,
        _ => 16*4,
    };

    // clear number of alien dead
    world.reset_alien_dead();
    // set music back to lowest tempo
    world.set_current_bpm(0);

    // set the top left position of swarm
    let alien_swarm_position = Point::new(
        ALIEN_TOP_LEFT_X_START_POSITION, 
        ALIEN_TOP_LEFT_Y_START_POSITION + vert_offset);
    *world.get_mut_alien_swarm_top_left_postion() = alien_swarm_position;

    // direction of swarm is going right to begin with
    let alien_swarm_direction = 1;
    *world.get_mut_alien_swarm_direction() = alien_swarm_direction;

    // reset alien movement speed
    *world.get_mut_alien_speed() = ALIEN_INITIAL_SPEED;
    
    // reset alien column counts
    for c in 0..world.get_number_alien_columns() {
        world.alien_columns[c] = NUMBER_ALIEN_ROWS as i32;
    }

    let mut pos = Point::new(ALIEN_TOP_LEFT_X_START_POSITION, ALIEN_TOP_LEFT_Y_START_POSITION + vert_offset);
    let transform = Vector::new(ALIEN_SPACING_HORZ, 0);

    for alien_index in 0..world.get_number_aliens() {
        if alien_index % NUMBER_ALIEN_COLUMNS == 0 {
            pos = Point::new(
                ALIEN_TOP_LEFT_X_START_POSITION, 
                ALIEN_TOP_LEFT_Y_START_POSITION + 
                vert_offset + ALIEN_SPACING_VERT*(alien_index/NUMBER_ALIEN_COLUMNS) as u32);
        }

        if let Some(entity) = world.get_mut_entity(world.get_alien(alien_index)) {
            if let Entity::Alien(alien) = entity {
                alien.position = pos;
                alien.bounding_box.origin = Point::new(0,0);
                alien.is_alive = true;
            }
        }

        // move down for next line of aliens
        pos += transform;
    }
}

/// Setup for next level, restoring/resetting/configuring barriers, player, and aliens
/// 
/// # Arguments
/// 
/// * `world` World to be updated for next level play
pub fn next_level(world: &mut World) {
    
    // increment current level
    world.current_level_inc();

    reset_explosions(world);
    reset_barriers(world);
    reset_player(false, world);
    reset_aliens(world.get_current_level(), world);
}

/// Setup for next level, restoring/resetting/configuring barriers, player, and aliens
/// 
/// # Arguments
/// 
/// * `world` World to be updated for next level play
pub fn new_game(world: &mut World) {
    
    // increment current level
    *world.get_mut_current_level() = 1;

    reset_explosions(world);
    reset_barriers(world);
    reset_player(true, world);
    reset_aliens(world.get_current_level(), world);
}