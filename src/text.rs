//! Description: 
//! 
//! Handle all things text, which in this case is actually not a huge amount. In truth this is the 
//! bare minimum needed for Space Invaders.
//! 
//! Currently supports sprites for each digit and sprites for 
//! SCORE<1>, SCORE<2>, HI-SCORE, and CREDIT.
//! 
//! Copyright © 2020 Benedict Gaster. All rights reserved.
//! 

use crate::math::*;
use crate::sprite_sheet::{SpriteSheet, SheetJSON, Sprite};
use crate::frame::{Frame};


#[derive(Debug, Clone)]
struct Digit {
    pub sprite: Sprite,
}

impl Digit {
    pub fn new(sprite: Sprite) -> Self {
        Digit {
            sprite,
        }
    }
}

pub struct Digits {
    digits: Vec<Digit>,
}

impl Digits {
    /// initialise digits with a given spritesheet
    pub fn new(sheet_json: &SheetJSON) -> Self {

        let mut digits = vec![];

        // simple add all the digits [0..9]
        let s = sheet_json.frames.get("text_0.png").unwrap();
        digits.push(Digit::new(Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32)));
        let s = sheet_json.frames.get("text_1.png").unwrap();
        digits.push(Digit::new(Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32)));
        let s = sheet_json.frames.get("text_2.png").unwrap();
        digits.push(Digit::new(Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32)));
        let s = sheet_json.frames.get("text_3.png").unwrap();
        digits.push(Digit::new(Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32)));
        let s = sheet_json.frames.get("text_4.png").unwrap();
        digits.push(Digit::new(Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32)));
        let s = sheet_json.frames.get("text_5.png").unwrap();
        digits.push(Digit::new(Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32)));
        let s = sheet_json.frames.get("text_6.png").unwrap();
        digits.push(Digit::new(Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32)));
        let s = sheet_json.frames.get("text_7.png").unwrap();
        digits.push(Digit::new(Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32)));
        let s = sheet_json.frames.get("text_8.png").unwrap();
        digits.push(Digit::new(Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32)));
        let s = sheet_json.frames.get("text_9.png").unwrap();
        digits.push(Digit::new(Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32)));

        Digits {
            digits,
        }
    }

    /// render a given digit [0..9] to framebuffer
    pub fn render<'a>(&self, digit: u32, pos: Point, sheet: &SpriteSheet, frame: &mut Frame<'a>) {
        self.digits[digit as usize % self.digits.len()].sprite.render(pos.x, pos.y, sheet, frame);

    }

    pub fn render_string<'a>(&self, str: String, pos: Point, sheet: &SpriteSheet, frame: &mut Frame<'a>) {
        for (i,c) in str.chars().enumerate() {
            let position = Point::new(pos.x + 60*i as u32, pos.y);
            match c {
                '0' => self.render(0, position, sheet, frame),
                '1' => self.render(1, position, sheet, frame),
                '2' => self.render(2, position, sheet, frame),
                '3' => self.render(3, position, sheet, frame),
                '4' => self.render(4, position, sheet, frame),
                '5' => self.render(5, position, sheet, frame),
                '6' => self.render(6, position, sheet, frame),
                '7' => self.render(7, position, sheet, frame),
                '8' => self.render(8, position, sheet, frame),
                '9' => self.render(9, position, sheet, frame),
                _ => { }
            }
        }
    }

    /// render a positive power of 10 number, padded to 4 digits
    pub fn render_num<'a>(&self, num: u32, pos: Point, sheet: &SpriteSheet, frame: &mut Frame<'a>) {
        let s: String = num.to_string();

        let mut front_str = "".to_string();

        // prefix zeros to make it 4 digits long
        if s.len() < 4 {
            for _ in 0..(4 - s.len()) {
                front_str.push('0');
            }
        }

        front_str.push_str(&s);

        self.render_string(front_str, pos, sheet, frame);
    }
}

#[derive(Debug, Clone)]
pub struct Score {
    pub score: Sprite,
    pub score_player1: Sprite,
    pub score_player2: Sprite,
    pub hi_score: Sprite,
    pub credit: Sprite,
}

impl Score {
    pub fn new(sheet_json: &SheetJSON) -> Self {
        
        let s = sheet_json.frames.get("score.png").unwrap();
        let score = Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32);

        let s = sheet_json.frames.get("score_1.png").unwrap();
        let score_player1 = Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32);

        let s = sheet_json.frames.get("score_2.png").unwrap();
        let score_player2 = Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32);
        
        let s = sheet_json.frames.get("hi_score.png").unwrap();
        let hi_score = Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32);

        let s = sheet_json.frames.get("credit.png").unwrap();
        let credit = Sprite::new(s.frame.x as u32, s.frame.y as u32, s.frame.w as u32, s.frame.h as u32);
        
        Score {
            score,
            score_player1,
            score_player2,
            hi_score,
            credit,
        }   
    }

    #[inline]
    pub fn render_player1<'a>(&self, pos: Point, sheet: &SpriteSheet, frame: &mut Frame<'a>) {
        self.score.render(pos.x, pos.y, sheet, frame);
        self.score_player1.render(pos.x+self.score.width+260, pos.y, sheet, frame);
    }

    #[inline]
    pub fn render_player2<'a>(&self, pos: Point, sheet: &SpriteSheet, frame: &mut Frame<'a>) {
        self.score.render(pos.x, pos.y, sheet, frame);
        self.score_player2.render(pos.x+self.score.width+260, pos.y, sheet, frame);
    }

    #[inline]
    pub fn render_hi_score<'a>(&self, pos: Point, sheet: &SpriteSheet, frame: &mut Frame<'a>) {
        self.hi_score.render(pos.x, pos.y, sheet, frame);
    }

    #[inline]
    pub fn render_credit<'a>(&self, pos: Point, sheet: &SpriteSheet, frame: &mut Frame<'a>) {
        self.credit.render(pos.x, pos.y, sheet, frame);
    }
}