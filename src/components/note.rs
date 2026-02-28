use bevy::prelude::*;

#[derive(Component)]
pub enum NoteTiming {
    Growing(Timer),
    Moving(Timer),
    Holding(Timer, f32),
}

#[derive(Component)]
pub enum NoteType {
    Tap(usize),                               // button
    TapHold(usize, (usize, usize)),           // button, (divider, length)
    Touch((usize, char)),                     // (value, group)
    TouchHold((usize, char), (usize, usize)), // (value, group), (divider, length)
    SlideStar(usize),
    // TODO: Implement SlideTrail completely
    SlideTrail,
}

#[derive(Component)]
pub enum HoldNoteElement{
    Head, 
    Body, // body length and time
    Tail,
}

#[derive(Component)]
pub enum TouchElement {
    Center, 
    Triangle
}
