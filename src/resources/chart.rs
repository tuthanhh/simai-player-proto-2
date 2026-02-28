use crate::parser::element::ChartEvent;
use bevy::prelude::*;
#[derive(Resource)]
pub struct ChartPlayback {
    pub events: Vec<ChartEvent>,
    pub current_index: usize,

    pub bpm: f32,
    pub resolution: u32,
    // Speed of the note
    pub note_speed: f32,
    // Playing speed
    pub chart_speed: f32,
    // Is playing
    pub is_playing: bool,
}

impl Default for ChartPlayback {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            current_index: 0,
            bpm: 240.0,
            resolution: 8,
            note_speed: 7.0,
            chart_speed: 1.0,
            is_playing: true,
        }
    }
}
