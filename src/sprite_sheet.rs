//! Description: 
//! 
//! 
//! Copyright © 2020 Benedict Gaster. All rights reserved.

use std::path::Path;
use std::fs::File;
use std::collections::HashMap;

use image::{DynamicImage, Rgba, ImageBuffer};

use serde::{Deserialize, Serialize};

use crate::frame::*;

//------------------------------------------------------------------------------
// Sprite sheet JSON representation
//------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XYWHJSON {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WHJSON {
    pub w: i32,
    pub h: i32,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteJSON {
    pub frame: XYWHJSON,
    pub rotated: bool,
    pub trimmed: bool,
    pub spriteSourceSize: XYWHJSON,
    pub sourceSize: WHJSON,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaJSON {
    pub app: String,
    pub version: String,
    pub image: String,
    pub format: String,
    pub size: WHJSON,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetJSON {
    pub frames: HashMap<String, SpriteJSON>,
    pub meta: MetaJSON,
}

impl SheetJSON {
    pub fn new<P>(jsonfile: P) -> Self
        where P: AsRef<Path> {
            let mut json = String::new();
            let mut file = File::open(jsonfile).expect("Unable open sprite sheet JSON file");
            serde_json::from_reader(file).expect("Invalid sprite sheet JSON file")
    }
}



// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct AnimationsJSON {
//     animations: ,
// }

//------------------------------------------------------------------------------
// Sprites
//------------------------------------------------------------------------------

pub enum BlendMode {
    Normal,             // O = sS
    Add,                // O = sS + dD
    Substract,          // O = sS - dD
    RevSubstract,       // O = dD - sS
    Min,                // O = min(Sr, Dr)
    Max,                // O = max(Sr, Dr)
}

#[derive(Debug, Clone)]
pub struct Sprite {
    /// x postion of sprite within sheet
    pub x: u32,
    /// y position of sprite within sheet
    pub y: u32,
    /// width of sprite
    pub width: u32,
    /// height of sprite
    pub height: u32,
}

pub type SpriteMask = Vec<Vec<u8>>;

pub fn print_sprite_mask(mask: &SpriteMask) {
    for row in mask {
        for col in row {
            print!("{}", col);
        }  
        println!("");
    }
}

impl Sprite {
    /// create sprite
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Sprite {
            x,
            y,
            width,
            height,
        }
    }

    /// create a mask of the sprite
    pub fn create_mask(&self, sheet: &SpriteSheet) -> SpriteMask {
        let mut mask = vec![];
        for sy in 0..=self.height {
            let mut row = vec![];
            for sx in 0..=self.width {
                let Rgba(rgba) = sheet.texture.get_pixel(sx+self.x,sy+self.y);
                if rgba[3] != 0 { 
                    // push 1 if alpha channel not 0
                    //print!("1");
                    row.push(1);
                }
                else {
                    // print!("0");
                    // push 0 is alpha channel 0
                    row.push(0);
                }
            }
            mask.push(row);
            // println!("");
        }

        // println!("-----------");
        // print_sprite_mask(&mask);

        mask
    }

    /// render sprite to frame with mask
    pub fn render_with_mask<'a>(&self, x: u32, y: u32, mask: &SpriteMask, sheet: &SpriteSheet, frame: &mut Frame<'a>) {
        let mut px = x;
        let mut py = y;

        for sy in 0..=self.height {
            for sx in 0..=self.width {
                let Rgba(rgba) = sheet.texture.get_pixel(sx+self.x,sy+self.y);
                
                // check mask not 0 
                // NOTE: no need to check alpha channel, as mask would be zero in that case
                if mask[sy as usize][sx as usize] != 0 { 
                    frame.put_pixel(px, py, rgba);
                    // clip if necessary
                    // if  px < frame.width*4 && py < frame.height*4 {
                    //     frame.frame[(px+py*frame.width*4) as usize]   = rgba[0];
                    //     frame.frame[(px+1+py*frame.width*4) as usize] = rgba[1];
                    //     frame.frame[(px+2+py*frame.width*4) as usize] = rgba[2];
                    //     frame.frame[(px+3+py*frame.width*4) as usize] = rgba[3];
                    // } 
                    //}
                }
                px += 4;
            }
            py += 1;
            px = x;
        }
    }

    /// render sprite to frame
    pub fn render<'a>(&self, x: u32, y: u32, sheet: &SpriteSheet, frame: &mut Frame<'a>) {
        let mut px = x;
        let mut py = y;

        for sy in 0..=self.height {
            for sx in 0..=self.width {
                let Rgba(rgba) = sheet.texture.get_pixel(sx+self.x,sy+self.y);
                // todo: ADD BLENDING MODE
                if rgba[3] != 0 { // check alpha channel not 0
                    frame.put_pixel(px, py, rgba);
                    //else {
                    // clip if necessary
                    // if  px < frame.width*4 && py < frame.height*4 {
                    //     // if rgba[0] == 0xe9 && rgba[1] == 0x1e && rgba[2] == 0x1e {
                    //     //     println!("0xED {}", rgba[3]);
                    //     //     // frame.frame[(px+py*frame.width*4) as usize]   = 0xFF;
                    //     //     // frame.frame[(px+1+py*frame.width*4) as usize] = 0xFF;
                    //     //     // frame.frame[(px+2+py*frame.width*4) as usize] = 0x0;
                    //     //     // frame.frame[(px+3+py*frame.width*4) as usize] = 0x0;
                    //     // }

                    //     frame.frame[(px+py*frame.width*4) as usize]   = rgba[0];
                    //     frame.frame[(px+1+py*frame.width*4) as usize] = rgba[1];
                    //     frame.frame[(px+2+py*frame.width*4) as usize] = rgba[2];
                    //     frame.frame[(px+3+py*frame.width*4) as usize] = rgba[3];
                    // } 
                    //}
                }
                px += 4;
            }
            py += 1;
            px = x;
        }
    }
}

//------------------------------------------------------------------------------
// Sprite sheets
//------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SpriteSheet {
    pub texture: ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>,
    pub width: u32,
    pub height: u32
}

impl SpriteSheet {
    pub fn new<P>(image_file: P) -> SpriteSheet
        where P: AsRef<Path> {

            let img : DynamicImage = image::open(image_file).unwrap();
            let tex = img.to_rgba();
            let dim = tex.dimensions();
            SpriteSheet {
                texture: tex,
                width: dim.0,
                height: dim.1,
            }
    }
}

//------------------------------------------------------------------------------
// Animation JSON
//------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationJSON {
    pub name: String,
    pub bullet: String,
    pub framerate: i32,
    pub frames: Vec<String>,
}

impl AnimationJSON {
    pub fn from_json<P>(jsonfile: P) -> HashMap<String, AnimationJSON>
        where P: AsRef<Path> {
            let file = File::open(jsonfile).expect("Unable open sprite sheet JSON file");
            serde_json::from_reader(file).expect("Invalid sprite sheet JSON file")
    }
}