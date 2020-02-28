//! Description: 
//! 
//! Very simple config file support, currenlty only used for high score.
//! 
//! Copyright © 2020 Benedict Gaster. All rights reserved.

use confy;

/// Configuration structure for space invaders, that is stored persistently (externally)
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// version number
    version: String,
    /// most recent highscore
    high_score: u32,
}

/// `Config` implements `Default`
impl ::std::default::Default for Config {
    fn default() -> Self { 
        Self { 
            version: "0.1".into(), 
            high_score: 0, 
        } 
    }
}

impl Config {
    /// Create (load) configutation 
    pub fn new() -> Self {
        confy::load("space-invaders").unwrap()
    }

    /// update highscore (does not store externally)
    /// 
    /// # Arguments
    /// 
    /// `score` Score to be stored
    pub fn udpate_highscore(&mut self, score: u32) {
        self.high_score = score;
    }

    /// returns the current highscore from configuration
    pub fn get_high_score(&self) -> u32 {
        self.high_score
    }

    /// returns the version number from configuration
    pub fn get_version(&self) -> String {
        self.version.clone()
    }

    /// stores configuration externally
    pub fn store(&self) {
        confy::store("space-invaders", self).unwrap();
    }
}