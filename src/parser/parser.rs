use super::element::*;
use regex::Regex;
use std::path::Path;

/// Parse a simai chart file and return a sequence of chart events
pub fn parse_chart(path: &Path) -> Result<Vec<ChartEvent>, std::io::Error> {
    println!("Parsing chart: {:?}", path);

    // Read file and remove whitespace
    let content = std::fs::read_to_string(path)?;
    let clean: String = content.chars().filter(|c| !c.is_whitespace()).collect();

    // Define regex patterns for handling bpm changes and resolution changes
    let bpm_regex = Regex::new(r"\((\d+(\.\d+)?)\)").unwrap();
    let res_regex = Regex::new(r"\{(\d+)\}").unwrap();

    // Split chart into tokens (comma-separated)
    let tokens: Vec<&str> = clean.split(',').collect();
    let mut events: Vec<ChartEvent> = Vec::new();

    // Process each token
    for (idx, token) in tokens.iter().enumerate() {
        let mut current_str = token.to_string();

        // Check for BPM change
        if let Some(captures) = bpm_regex.captures(&current_str) {
            if let Ok(bpm) = captures[1].parse::<f32>() {
                events.push(ChartEvent::BpmChange(bpm));
                current_str = current_str[captures[0].len()..].to_string();
            }
        }

        // Check for resolution change
        if let Some(captures) = res_regex.captures(&current_str) {
            if let Ok(resolution) = captures[1].parse::<u32>() {
                events.push(ChartEvent::ResolutionChange(resolution));
                current_str = current_str[captures[0].len()..].to_string();
            }
        }
        if current_str.is_empty() {
            events.push(ChartEvent::Rest);
            continue;
        }
        // Parse remaining notes (if any)
        if !current_str.is_empty() {
            let notes_result: Result<Vec<Note>, String> = current_str
                .split('/')
                .map(parse_note)
                .collect::<Result<Vec<Vec<Note>>, String>>()
                .map(|v| v.into_iter().flatten().collect());

            match notes_result {
                Ok(notes) => events.push(ChartEvent::NoteGroup(notes)),
                Err(e) => eprintln!(
                    "Warning: Error parsing note at token {}: '{}' - {}",
                    idx, token, e
                ),
            }
        }
    }

    println!("Parsed {} events", events.len());
    Ok(events)
}

pub fn parse_note(note_str: &str) -> Result<Vec<Note>, String> {
    let note_re = Regex::new(
        r"(?x)
            ^([A-E][1-8]|[1-8]|[1-8][1-8]|C){1}
            ([xhfb]*)
            (?:\[(\d+(?:\.\d+)?):(\d+(?:\.\d+)?)\])?
            ((?:pp|qq|[-^v<pqsVzw>]).*)?
            $
        ",
    )
    .map_err(|_| "Regex failed")?;
    let caps = note_re
        .captures(note_str)
        .ok_or_else(|| format!("Invalid note syntax: '{}'", note_str))?;
    let raw_loc = &caps[1];

    let modifiers = &caps[2];
    let is_break = modifiers.contains('x');
    let is_firework = modifiers.contains('f');

    let is_hold = modifiers.contains('h');
    let hold_duration = if let (Some(divider), Some(length)) = (caps.get(3), caps.get(4)) {
        Some((
            divider.as_str().parse().unwrap_or(0),
            length.as_str().parse().unwrap_or(1),
        ))
    } else {
        None
    };
    let is_slide = caps.get(5).is_some();

    if raw_loc.len() == 2 && raw_loc.chars().all(|c| c.is_ascii_digit()) {
        // Two-digit button press (e.g., "12" for buttons 1 and 2)
        let chars: Vec<char> = raw_loc.chars().collect();
        let btn1 = chars[0].to_digit(10).unwrap_or(0) as usize;
        let btn2 = chars[1].to_digit(10).unwrap_or(0) as usize;
        return Ok(vec![
            Note {
                is_break,
                is_firework,
                note_type: NoteDetail::Tap(btn1),
            },
            Note {
                is_break,
                is_firework,
                note_type: NoteDetail::Tap(btn2),
            },
        ]);
    } else if let Ok(btn_num) = raw_loc.parse::<usize>() {
        if is_slide {
            return Ok(vec![Note {
                is_break,
                is_firework,
                note_type: NoteDetail::SlideStar(btn_num),
            }]);
        } else if is_hold {
            return Ok(vec![Note {
                is_break,
                is_firework,
                note_type: NoteDetail::TapHold(btn_num, hold_duration.unwrap_or((1, 0))),
            }]);
        } else {
            return Ok(vec![Note {
                is_break,
                is_firework,
                note_type: NoteDetail::Tap(btn_num),
            }]);
        }
    } else {
        // It's a touch note (A1, C, etc.)
        let chars: Vec<char> = raw_loc.chars().collect();
        let zone = chars[0].to_ascii_uppercase();
        // If there's a number (A1), parse it. If not (C, E), default to 0 or specific logic
        let index = if chars.len() > 1 {
            chars[1].to_digit(10).unwrap_or(0) as usize
        } else {
            1 // C usually doesn't have indices, or implies index 1
        };
        if is_hold {
            return Ok(vec![Note {
                is_break,
                is_firework,
                note_type: NoteDetail::TouchHold((index, zone), hold_duration.unwrap_or((1, 0))),
            }]);
        } else {
            return Ok(vec![Note {
                is_break,
                is_firework,
                note_type: NoteDetail::Touch((index, zone)),
            }]);
        }
    };
}
