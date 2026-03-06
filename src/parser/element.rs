// The main events found in the chart stream
#[derive(Debug)]
pub enum ChartEvent {
    BpmChange(f32),        // (191)
    ResolutionChange(u32), // {8}
    NoteGroup(Vec<Note>),  // A group of notes played simultaneously (separated by /)
    Rest,                  // An empty comma with no notes
}

#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub is_break: bool,
    pub is_firework: bool,
    pub note_type: NoteDetail, // Renamed from 'type'
}

#[derive(Debug, Clone, Copy)]
pub enum NoteDetail {
    Tap(usize),                               // button
    TapHold(usize, (usize, usize)),           // button, (divider, length)
    Touch((usize, char)),                     // (value, group)
    TouchHold((usize, char), (usize, usize)), // (value, group), (divider, length)
    Slide(usize, SlideShape, (usize, usize)), // value, shape, (divider, length)
}

#[derive(Debug, Clone, Copy)]
pub enum SlideShape {
    Straight(usize, usize),                 // start, end
    ShortArc(usize, usize),                 // start, end
    ClockwiseArc(usize, usize),             // start, end
    CounterClockwiseArc(usize, usize),      // start, end
    VShape(usize, usize),                   // start, end
    PShape(usize, usize),                   // start, end
    QShape(usize, usize),                   // start, end
    GrandVShape(usize, usize, usize),       // start, end, mid
    GrandPShape(usize, usize),              // start, end
    GrandQShape(usize, usize),              // start, end
    Thunderbolt(usize, usize, bool),        // start, end, is_z
    FanShape(usize, (usize, usize, usize)), // start, (end 1, end 2, end 3)
}
