use bevy::prelude::*;

#[derive(Component)]
pub enum NoteTiming {
    Growing(Timer),
    Moving(Timer),
    Holding(Timer, f32),
    Waiting(Timer),
    Sliding(Timer),
}

#[derive(Component)]
pub enum NoteType {
    Tap(usize),                               // button
    TapHold(usize, (usize, usize)),           // button, (divider, length)
    Touch((usize, char)),                     // (value, group)
    TouchHold((usize, char), (usize, usize)), // (value, group), (divider, length)
    Slide(usize, (usize, usize)),             // (button), (divider, length)
}

#[derive(Component)]
pub enum HoldNoteElement {
    Head,
    Body, // body length and time
    Tail,
}

#[derive(Component)]
pub enum TouchElement {
    Center,
    Triangle,
}

// Stores the calculated path so the Sliding phase knows where to go
#[derive(Component)]
pub struct SlidePath {
    pub waypoints: Vec<Vec2>,
    pub total_length: f32,
    pub track_entity: Option<Entity>, 
}

// Attached to each visual Chevron
#[derive(Component)]
pub struct SlideArrow {
    pub distance_along_path: f32,
}
