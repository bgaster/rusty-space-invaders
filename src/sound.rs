//! Description: 
//! 
//! Handle sound playback. All very simply sound effects and the music for the inpending 
//! invasion.
//! 
//! Copyright © 2020 Benedict Gaster. All rights reserved.

extern crate rodio;

use std::sync::Arc;
use std::io::Read;
use std::io;
use rodio::{Device, Sink, Source};

/// Static sound data stored in memory.
/// It is `Arc`'ed, so cheap to clone.
#[derive(Clone, Debug)]
pub struct SoundData(Arc<[u8]>);

impl SoundData {
    /// Load the file at the given path and create a new `SoundData` from it.
    pub fn new(str: &str) -> Self {
        let mut reader = std::fs::File::open(str).unwrap();

        SoundData::from_read(&mut reader)
    }

    /// Copies the data in the given slice into a new `SoundData` object.
    pub fn from_bytes(data: &[u8]) -> Self {
        SoundData(Arc::from(data))
    }

    /// Creates a `SoundData` from any `Read` object; this involves
    /// copying it into a buffer.
    pub fn from_read<R>(reader: &mut R) -> Self
    where
        R: Read,
    {
        let mut buffer = Vec::new();
        let _ = reader.read_to_end(&mut buffer).unwrap();

        SoundData::from(buffer)
    }

    /// Indicates if the data can be played as a sound.
    pub fn can_play(&self) -> bool {
        let cursor = io::Cursor::new(self.clone());
        rodio::Decoder::new(cursor).is_ok()
    }
}

impl From<Arc<[u8]>> for SoundData {
    #[inline]
    fn from(arc: Arc<[u8]>) -> Self {
        SoundData(arc)
    }
}

impl From<Vec<u8>> for SoundData {
    fn from(v: Vec<u8>) -> Self {
        SoundData(Arc::from(v))
    }
}

impl From<Box<[u8]>> for SoundData {
    fn from(b: Box<[u8]>) -> Self {
        SoundData(Arc::from(b))
    }
}

impl AsRef<[u8]> for SoundData {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}


pub struct Sound {
    /// output device for audio playback
    output_device: Device,
    // give effects their own sinks so they can happen concurrently
    /// sink for playing effect
    player_explosion_sink: Sink,
    /// sink for play alien explosion effect
    alien_explosion_sink: Sink,
    /// sink for player bullet
    player_shot_sink: Sink,
    /// sink for playing sound track
    music_sink: Vec<Sink>,
    current_bpm: usize,

    // sound effects
    player_shot_data: io::Cursor<SoundData>,
    player_explosion_data: io::Cursor<SoundData>,
    alien_explosion_data: io::Cursor<SoundData>,
}

impl Sound {
    pub fn new(
        player_shot: &str,
        player_explosion: &str,
        alien_explosion: &str,
        music: Vec<&str>) -> Self{

        let player_shot = SoundData::new(player_shot);
        let player_explosion = SoundData::new(player_explosion);
        let alien_explosion = SoundData::new(alien_explosion);
    
        let mut music_sinks = vec![];

        let device = rodio::default_output_device().unwrap();
        let player_explosion_sink = Sink::new(&device);
        let player_shot_sink = Sink::new(&device);
        
        let alien_explosion_sink = Sink::new(&device);

        for s in music {
            let music = SoundData::new(s);
            let music = rodio::Decoder::new(io::Cursor::new(music)).unwrap()
                .repeat_infinite().
                speed(1.0);

            let music_sink   = Sink::new(&device);
            music_sink.append(music);
            music_sink.pause();
            music_sinks.push(music_sink);
        }

        // lower the sounds of effects, compared to the music
        // should really do this in ableton
        player_shot_sink.set_volume(0.1);
        alien_explosion_sink.set_volume(0.1);
        player_explosion_sink.set_volume(0.1);

        Sound {
            // TODO: check devices and so on
            output_device: device,
            player_explosion_sink,
            alien_explosion_sink,
            player_shot_sink,
            music_sink: music_sinks,
            current_bpm: 0,
            player_shot_data: io::Cursor::new(player_shot),
            player_explosion_data: io::Cursor::new(player_explosion),
            alien_explosion_data: io::Cursor::new(alien_explosion),
        }
    }

    /// play alien marching music
    /// 
    /// If there is music currenlty playing, then it is paused, and the new
    /// bpm is started
    /// 
    /// # Arguments 
    /// 
    /// * `index`: index of music to play
    pub fn play_music(&mut self, bpm: usize) {
        // force to be inbounds, avoiding any panics
        if bpm < self.music_sink.len() {
            self.music_sink[self.current_bpm].pause();
            self.music_sink[bpm].play();
            self.current_bpm = bpm;
        }
    }

    /// returns the number of different music speed variants
    pub fn number_of_music_variants(&self) -> usize {
        self.music_sink.len()
    }

    /// pause current music if playing, if no music is playing nothing is changed
    pub fn pause_music(&self) {
        self.music_sink[self.current_bpm].pause();
    }

    /// play sound for player's shot
    pub fn play_player_shot(&self) {
        if self.player_shot_sink.empty() {
            let sound = rodio::Decoder::new(self.player_shot_data.clone()).unwrap();
            self.player_shot_sink.append(sound);
        }
    }

    /// play sound for player's explosion
    pub fn play_player_explosion(&self) {
        let sound = rodio::Decoder::new(self.player_explosion_data.clone()).unwrap();
        self.player_explosion_sink.append(sound);
    }

    /// play sound for player's explosion
    pub fn play_alien_explosion(&self) {
            let sound = rodio::Decoder::new(self.alien_explosion_data.clone()).unwrap();
            self.alien_explosion_sink.append(sound);
            
    }
}