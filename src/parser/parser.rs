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
    if note_str.is_empty() {
        return Err("Empty note string".to_string());
    }

    // Check if this is a slide by looking for slide patterns
    let slide_pattern_re = Regex::new(r"(pp|qq|[-^v<>pqszVw*])").unwrap();
    let has_slide = slide_pattern_re.is_match(note_str);
    
    if has_slide {
        return parse_slide_note(note_str);
    }

    // Otherwise parse as regular note (tap or hold)
    parse_tap_or_touch_note(note_str)
}

fn parse_tap_or_touch_note(note_str: &str) -> Result<Vec<Note>, String> {
    // Regex for tap/touch notes with optional hold
    let note_re = Regex::new(
        r"(?x)
            ^([A-E][1-8]|[1-8]|[1-8][1-8]|C|E)  # Button or touch location
            ([xhfb]*)                            # Modifiers
            (?:\[(\d+):(\d+)\])?                 # Optional hold duration
            ([xhfb]*)                            # Optional modifiers after duration
            $
        ",
    )
    .map_err(|_| "Regex compilation failed")?;
    
    let caps = note_re
        .captures(note_str)
        .ok_or_else(|| format!("Invalid note syntax: '{}'", note_str))?;
    
    let raw_loc = &caps[1];
    let modifiers = &caps[2];
    let modifiers_after = caps.get(5).map(|m| m.as_str()).unwrap_or("");
    
    let is_break = modifiers.contains('x') || modifiers_after.contains('x');
    let is_firework = modifiers.contains('f') || modifiers_after.contains('f');
    let is_hold = modifiers.contains('h');
    
    let hold_duration = if let (Some(divider), Some(length)) = (caps.get(3), caps.get(4)) {
        Some((
            divider.as_str().parse().unwrap_or(1),
            length.as_str().parse().unwrap_or(1),
        ))
    } else {
        None
    };

    // Handle two-digit button press (e.g., "12" for buttons 1 and 2)
    if raw_loc.len() == 2 && raw_loc.chars().all(|c| c.is_ascii_digit()) {
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
    }
    
    // Handle regular button number
    if let Ok(btn_num) = raw_loc.parse::<usize>() {
        if is_hold {
            let duration = hold_duration.ok_or("Hold requires duration")?;
            return Ok(vec![Note {
                is_break,
                is_firework,
                note_type: NoteDetail::TapHold(btn_num, duration),
            }]);
        } else {
            return Ok(vec![Note {
                is_break,
                is_firework,
                note_type: NoteDetail::Tap(btn_num),
            }]);
        }
    }
    
    // Handle touch note (A1, C, E, E2, etc.)
    let chars: Vec<char> = raw_loc.chars().collect();
    let zone = chars[0].to_ascii_uppercase();
    let index = if chars.len() > 1 {
        chars[1].to_digit(10).unwrap_or(1) as usize
    } else {
        1 // Default index for single-letter zones like C or E
    };
    
    if is_hold {
        let duration = hold_duration.unwrap_or((1, 1));
        return Ok(vec![Note {
            is_break,
            is_firework,
            note_type: NoteDetail::TouchHold((index, zone), duration),
        }]);
    } else {
        return Ok(vec![Note {
            is_break,
            is_firework,
            note_type: NoteDetail::Touch((index, zone)),
        }]);
    }
}

fn parse_slide_note(note_str: &str) -> Result<Vec<Note>, String> {
    // Parse a note with slide patterns
    // Format: <button><modifiers><slide_pattern><target>[duration]<modifiers>
    // Examples: 1-5[8:1], 3x>6b[16:9], 5V71[4:1], 1-4[8:1]*-6[8:1]
    
    let note_re = Regex::new(
        r"(?x)
            ^([1-8])                                 # Start button (must be digit for slides)
            ([xfb]*)                                 # Modifiers before slide
            ((?:pp|qq|[-^v<>pqszVw])\d+              # Slide pattern + target
             (?:(?:pp|qq|[-^v<>pqszVw])\d+)*         # Optional additional shape+target (like p4>6)
             (?:-\d+)*                               # Optional chained targets (like 8-4-7)
             (?:[xfb]*)                              # Optional modifiers after target
             (?:\[[\d:]+\])?                         # Optional duration
             (?:[xfb]*)                              # Optional modifiers after duration
             (?:[*](?:pp|qq|[-^v<>pqszVw])\d+       # Optional star-chained slide
               (?:[xfb]*)                            # Optional modifiers after chained target
               (?:\[[\d:]+\])?                       # Optional chained duration
               (?:[xfb]*))*                          # Optional modifiers after chained slide
            )
            ([xfb]*)                                 # Modifiers at end
            $
        ",
    )
    .map_err(|_| "Slide regex compilation failed")?;
    
    let caps = note_re
        .captures(note_str)
        .ok_or_else(|| format!("Invalid slide syntax: '{}'", note_str))?;
    
    let start_btn = caps[1].parse::<usize>().unwrap();
    let modifiers_before = &caps[2];
    let slide_pattern_str = &caps[3];
    let modifiers_after = &caps[4];
    
    let is_break = modifiers_before.contains('x') || modifiers_after.contains('x');
    let is_firework = modifiers_before.contains('f') || modifiers_after.contains('f');
    
    // Check if this is a star-chained slide (multiple sequential slides)
    if slide_pattern_str.contains('*') {
        return parse_star_chained_slides(start_btn, slide_pattern_str, is_break, is_firework);
    }
    
    // Parse single slide pattern
    let slide_shape = parse_slide_pattern(start_btn, slide_pattern_str)?;
    
    Ok(vec![Note {
        is_break,
        is_firework,
        note_type: slide_shape,
    }])
}

fn parse_slide_pattern(start_btn: usize, pattern_str: &str) -> Result<NoteDetail, String> {
    // Parse individual slide segment(s)
    // Format: <shape><target>[duration] or <shape><target>-<target>[duration] (chained targets)
    //     or: <shape><target><shape><target>[duration] (like p4>6 - two shapes in sequence)
    //     or: V<mid><end>[duration] (Grand V-shape with mid button)
    
    // First, extract the duration (always at the end before optional modifiers)
    let duration_re = Regex::new(r"\[(\d+):(\d+)\]").unwrap();
    let duration = if let Some(cap) = duration_re.captures(pattern_str) {
        (
            cap[1].parse::<usize>().unwrap_or(1),
            cap[2].parse::<usize>().unwrap_or(1),
        )
    } else {
        return Err(format!("Slide requires duration: '{}'", pattern_str));
    };
    
    // Remove duration and trailing modifiers from pattern
    let pattern_without_duration = duration_re.replace(pattern_str, "");
    let pattern_clean = pattern_without_duration.trim_end_matches(|c| c == 'x' || c == 'f' || c == 'b');
    
    // Check for star notation (chained slides like *-6 or *V71)
    if pattern_clean.starts_with('*') {
        // Star notation means continue from previous slide's end
        // For now, we'll parse it as if the start button is correct
        let after_star = &pattern_clean[1..];
        return parse_slide_pattern(start_btn, &format!("{}{}", after_star, duration_re.find(pattern_str).unwrap().as_str()));
    }
    
    // Check for Grand V-shape (V<mid><end>) first - this needs special handling
    let grand_v_re = Regex::new(r"^V(\d)(\d)$").unwrap();
    if let Some(caps) = grand_v_re.captures(pattern_clean) {
        let mid_btn = caps[1].parse::<usize>().unwrap();
        let end_btn = caps[2].parse::<usize>().unwrap();
        return Ok(NoteDetail::Slide(
            start_btn,
            SlideShape::GrandVShape(start_btn, end_btn, mid_btn),
            duration,
        ));
    }
    
    // Check for dash-chained straight slides (like -4-7 meaning 8->4->7)
    // Use GrandVShape to store the waypoint (mid parameter)
    let dash_chain_re = Regex::new(r"^-(\d+)-(\d+)$").unwrap();
    if let Some(caps) = dash_chain_re.captures(pattern_clean) {
        let waypoint = caps[1].parse::<usize>().unwrap();
        let end_btn = caps[2].parse::<usize>().unwrap();
        return Ok(NoteDetail::Slide(
            start_btn,
            SlideShape::GrandVShape(start_btn, end_btn, waypoint),
            duration,
        ));
    }
    
    // Try to match multiple shape segments (like p4>6)
    // Match shape indicators but exclude colons and digits
    let multi_segment_re = Regex::new(
        r"(pp|qq|[-^v<>pqszVw]+)(\d+)"
    ).unwrap();
    
    let segments: Vec<_> = multi_segment_re.captures_iter(pattern_clean).collect();
    
    if segments.is_empty() {
        return Err(format!("No valid slide pattern found: '{}'", pattern_clean));
    }
    
    // For now, use the last segment as the main slide
    // TODO: Properly handle multi-segment slides by creating composite shapes
    let last_segment = segments.last().unwrap();
    let shape_indicator = last_segment.get(1).unwrap().as_str();
    let end_btn = last_segment.get(2)
        .unwrap()
        .as_str()
        .parse::<usize>()
        .map_err(|_| format!("Invalid target button: '{}'", last_segment.get(2).unwrap().as_str()))?;
    
    let shape = match shape_indicator {
        "-" => SlideShape::Straight(start_btn, end_btn),
        "^" => SlideShape::ShortArc(start_btn, end_btn),
        "v" => SlideShape::VShape(start_btn, end_btn),
        "V" => SlideShape::VShape(start_btn, end_btn), // Single-segment V is regular V-shape
        "<" => SlideShape::CounterClockwiseArc(start_btn, end_btn),
        ">" => SlideShape::ClockwiseArc(start_btn, end_btn),
        "p" => SlideShape::PShape(start_btn, end_btn),
        "q" => SlideShape::QShape(start_btn, end_btn),
        "pp" => SlideShape::GrandPShape(start_btn, end_btn),
        "qq" => SlideShape::GrandQShape(start_btn, end_btn),
        "s" => SlideShape::Thunderbolt(start_btn, end_btn, false),
        "z" => SlideShape::Thunderbolt(start_btn, end_btn, true),
        "w" => {
            // Fan shape - requires multiple targets
            // For now treating as straight, needs special parsing
            SlideShape::FanShape(start_btn, (end_btn, (end_btn + 1) % 8, (end_btn - 1) % 8))
        }
        "*" => {
            // Star notation for chained slides
            // This should continue from previous slide's end position
            SlideShape::Straight(start_btn, end_btn)
        }
        _ => return Err(format!("Unknown slide shape: {}", shape_indicator)),
    };
    
    Ok(NoteDetail::Slide(start_btn, shape, duration))
}

/// Parse star-chained slides like "1-4[8:1]*-6[8:1]" into multiple sequential notes
fn parse_star_chained_slides(start_btn: usize, pattern_str: &str, is_break: bool, is_firework: bool) -> Result<Vec<Note>, String> {
    // Split by star to get individual slide segments
    let segments: Vec<&str> = pattern_str.split('*').collect();
    let mut notes = Vec::new();
    let mut current_start = start_btn;
    
    for segment in segments {
        let note_detail = parse_slide_pattern(current_start, segment)?;
        
        // Extract end button from the slide shape to use as next start
        current_start = match note_detail {
            NoteDetail::Slide(_, shape, _) => {
                match shape {
                    SlideShape::Straight(_, end) => end,
                    SlideShape::ShortArc(_, end) => end,
                    SlideShape::ClockwiseArc(_, end) => end,
                    SlideShape::CounterClockwiseArc(_, end) => end,
                    SlideShape::VShape(_, end) => end,
                    SlideShape::PShape(_, end) => end,
                    SlideShape::QShape(_, end) => end,
                    SlideShape::GrandVShape(_, end, _) => end,
                    SlideShape::GrandPShape(_, end) => end,
                    SlideShape::GrandQShape(_, end) => end,
                    SlideShape::Thunderbolt(_, end, _) => end,
                    SlideShape::FanShape(_, (end1, _, _)) => end1,
                }
            }
            _ => return Err("Star-chained segment is not a slide".to_string()),
        };
        
        notes.push(Note {
            is_break,
            is_firework,
            note_type: note_detail,
        });
    }
    
    Ok(notes)
}
