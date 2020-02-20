//! Description: 
//! 
//! 
//! Copyright © 2020 Benedict Gaster. All rights reserved.

use euclid::{Point2D};

use crate::math::*;

use crate::sprite_sheet::{AnimationJSON, SheetJSON, Sprite, SpriteSheet};
use crate::frame::{Frame};

#[derive(Debug, Clone)]
pub struct Animation {
    framerate: u32,
    current: usize,
    rate: u32,
    bounding_box: Rect,
    sprites: Vec<Sprite>,
}

impl Animation {
    pub fn new(
        ani_json: &AnimationJSON, 
        sheet_json: &SheetJSON) -> Self {
        let mut animation = Animation {
            framerate: ani_json.framerate as u32,
            current: 0,
            rate: 0,
            bounding_box: Rect::default(),
            sprites: vec![],
        };

        for n in &ani_json.frames {
            let s = sheet_json.frames.get(n).unwrap();
            animation.sprites.push(
                Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32));
            animation.bounding_box.size = Size::new(s.frame.w as u32, s.frame.h as u32);
        }

        animation
    }

    pub fn get_bounding_box(&self) -> Rect {
        self.bounding_box
    } 

    pub fn step(&mut self) {
        if self.rate == self.framerate {
            self.current = (self.current + 1) % self.sprites.len();
            self.rate = 0;
        }
        else {
            self.rate += 1;
        }
    }

    pub fn render<'a>(&self, pos: Point2D<u32,u32>, sheet: &SpriteSheet, frame: &mut Frame<'a>) {
        self.sprites[self.current].render(pos.x, pos.y, sheet, frame);
    }
}