// The main events found in the chart stream
#[derive(Debug)]
pub enum ChartEvent {
    BpmChange(f32),        // (191)
    ResolutionChange(u32), // {8}
    NoteGroup(Vec<Note>),  // A group of notes played simultaneously (separated by /)
    Rest,                  // An empty comma with no notes
}

#[derive(Debug)]
pub struct Note {
    pub is_break: bool,
    pub is_firework: bool,
    pub note_type: NoteDetail, // Renamed from 'type'
}

#[derive(Debug)]
pub enum NoteDetail {
    Tap(usize), // button
    TapHold(usize, (usize, usize)), // button, (divider, length)
    Touch((usize, char)),     // (value, group)
    TouchHold((usize, char), (usize, usize)), // (value, group), (divider, length)
    SlideStar(usize),
    // TODO: Implement Slide completely.
}
