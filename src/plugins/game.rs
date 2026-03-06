#![allow(unused)]
use crate::{
    parser::{
        element::{
            ChartEvent::{self, *},
            Note, NoteDetail,
        },
        parser,
    },
    prelude::{note::*, sound::SoundResources, visual::NoteAssets, *},
    resources::chart::ChartPlayback,
};
use bevy::{
    app::App, color::palettes::css::LIGHT_CYAN, post_process::dof::calculate_focal_length,
    prelude::*,
};
use bevy_kira_audio::prelude::*;
use bevy_prototype_lyon::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI, SQRT_2};
use std::path::Path;
use std::time::Duration;

pub(crate) fn plugin(app: &mut App) {
    if !app.is_plugin_added::<ShapePlugin>() {
        app.add_plugins(ShapePlugin);
    }
    // Your game logic here
    app.init_resource::<ChartPlayback>()
        .init_resource::<SoundResources>()
        .add_systems(Startup, (spawn_judgement_ring, chart_setup, resource_setup))
        .add_systems(Update, (next_bar_process, update_note));
}
fn resource_setup(
    mut sound_resources: ResMut<SoundResources>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
) {
    let size = Vec2::new(window.width(), window.height());
    let radius = size.min_element() * 0.05;
    let hex_flat_width = radius * 2.0; // sqrt(3)

    // Initialize your sound resources here
    sound_resources.hit = asset_server.load("audio/SE_GAME_ANSWER_.wav");
    // 1. Define materials
    let tap_material = materials.add(ColorMaterial::from(Color::srgb(1.0, 0.4, 0.6))); // Pink
    let hold_material = materials.add(ColorMaterial::from(Color::srgb(1.0, 0.4, 0.6))); // Cyan
    let slide_material = materials.add(ColorMaterial::from(Color::srgb(0.4, 0.8, 1.0))); // Purple
    let paired_material = materials.add(ColorMaterial::from(Color::srgb(1.0, 1.0, 0.0))); // Yellow

    let tap_mesh = meshes.add(Annulus::new(radius, radius * 0.75).mesh().resolution(256));
    let hold_mesh = meshes.add(create_hollow_arch(radius));
    let touch_circle_mesh = meshes.add(Circle::new(radius * 0.1));
    let width = 8.0;
    let height = 20.0;
    let chevron_shape = ShapePath::new()
        .move_to(Vec2::new(-width, height))
        .line_to(Vec2::new(0.0, 0.0))
        .line_to(Vec2::new(-width, -height))
        .line_to(Vec2::new(0.0, -height))
        .line_to(Vec2::new(width, 0.0))
        .line_to(Vec2::new(0.0, height))
        .close();
    // 1. Define your base width and your desired border thickness
    let w = radius * std::f32::consts::SQRT_2 / 2.0;
    let t = radius * 0.15; // Change 0.1 to make the border thicker or thinner!

    let touch_triangle_mesh = meshes.add(Ring {
        outer_shape: Triangle2d::new(vec2(0.0, 0.0), vec2(w, w), vec2(-w, w)),
        inner_shape: Triangle2d::new(
            // Shift the tip straight up
            vec2(0.0, t * std::f32::consts::SQRT_2),
            // Shift the right corner down and left
            vec2(w - t * (1.0 + std::f32::consts::SQRT_2), w - t),
            // Shift the left corner down and right
            vec2(-(w - t * (1.0 + std::f32::consts::SQRT_2)), w - t),
        ),
    });
    let slide_mesh = meshes.add(create_hollow_star_mesh(radius, 0.60, 0.35 * radius, 5));
    let hold_body_mesh = meshes.add(Ring {
        outer_shape: Rectangle::new(hex_flat_width, 1.0),
        inner_shape: Rectangle::new(hex_flat_width * 0.75, 1.0),
    });
    // 3. Insert the resource into the world
    commands.insert_resource(NoteAssets {
        tap_mesh,
        hold_mesh,
        hold_body_mesh,
        slide_mesh,
        touch_circle_mesh,
        touch_triangle_mesh,
        chevron_shape,
        tap_material,
        hold_material,
        slide_material,
        paired_material,
    });
}
fn spawn_judgement_ring(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
) {
    let size = Vec2::new(window.width(), window.height());

    // Calculate the radius for the main ring
    let radius = size.min_element() / 2.0 - 40.0;

    // Spawn the main Annulus (ring)
    let shape = Annulus::new(radius, radius * 0.995).mesh().resolution(256);
    let mesh = meshes.add(shape);
    let material = materials.add(ColorMaterial::from(Color::WHITE));

    commands.spawn((Mesh2d(mesh), MeshMaterial2d(material.clone())));

    // Prepare the dot mesh
    let dot_size = radius * 0.02;
    let shape = Circle::new(dot_size).mesh().resolution(32);
    let dot_mesh = meshes.add(shape);

    // Loop 8 times to spawn 8 dots
    for i in 0..8 {
        // Calculate the angle for this specific dot (in radians)
        let angle = (i as f32 + 0.5) * (2.0 * PI / 8.0);

        // Use trigonometry to find the x and y coordinates on the circle
        let x = radius * angle.cos();
        let y = radius * angle.sin();

        commands.spawn((
            Mesh2d(dot_mesh.clone()),
            MeshMaterial2d(material.clone()),
            // Position the dot at the calculated x, y.
            // We set z to 1.0 so it renders on top of the ring!
            Transform::from_xyz(x, y, 1.0),
        ));
    }
}

fn chart_setup(mut chart: ResMut<ChartPlayback>) {
    chart.events = parser::parse_chart(Path::new("songs/M@GICAL☆CURE! LOVE ♥ SHOT!/maidata.txt"))
        .expect("Failed to parse chart");
    // or
    // chart.events = vec![ChartEvent::NoteGroup(vec![Note {
    //     is_break: false,
    //     is_firework: false,
    //     note_type: NoteDetail::Touch((1, 'B')),
    // }])]
}

pub fn next_bar_process(
    mut commands: Commands,
    mut chart: ResMut<ChartPlayback>,
    time: Res<Time>,
    note_assets: Res<NoteAssets>,
    mut timer: Local<Timer>,
    window: Single<&Window>,
) {
    if !chart.is_playing || chart.current_index >= chart.events.len() {
        chart.is_playing = true;
        return;
    }

    timer.tick(time.delta());

    if !timer.just_finished() && chart.current_index > 0 {
        return;
    }

    let size = Vec2::new(window.width(), window.height());
    let radius = size.min_element() * 0.05;
    let judgment_radius = size.min_element() / 2.0 - 40.0;

    // 3. Set the spawn distance from the center (e.g., 10% of screen size)
    let spawn_radius = size.min_element() * 0.12;

    while chart.current_index < chart.events.len() {
        let current_play = &chart.events[chart.current_index];
        let mut waiting = false;
        match current_play {
            ChartEvent::BpmChange(new_value) => {
                chart.bpm = *new_value;
            }
            ChartEvent::ResolutionChange(new_value) => {
                chart.resolution = *new_value;
            }
            ChartEvent::NoteGroup(notes) => {
                waiting = true;
                let is_paired = notes.len() >= 2;
                for note in notes {
                    // A. Determine the target button ID to calculate rotation
                    let button_id = match note.note_type {
                        NoteDetail::Tap(id)
                        | NoteDetail::TapHold(id, _)
                        | NoteDetail::Slide(id, _, _) => id,
                        _ => 0, // Fallback for Touches which might use a different coordinate system
                    };

                    let touch_id = match note.note_type {
                        NoteDetail::Touch((id, group)) | NoteDetail::TouchHold((id, group), _) => {
                            (id, group)
                        }
                        _ => (0, 'C'),
                    };

                    // B. Calculate position and angle
                    let button_angle = FRAC_PI_2 - (-0.5 + button_id as f32) * (2.0 * PI / 8.0);
                    let button_spawn_pos = polar_to_cartesian(spawn_radius, button_angle);
                    let button_note_transform =
                        Transform::from_translation(button_spawn_pos.extend(2.0))
                            .with_rotation(Quat::from_rotation_z(button_angle - PI / 2.0))
                            .with_scale(Vec3::ZERO);

                    let mut touch_angle = FRAC_PI_2 - (-0.5 + touch_id.0 as f32) * (2.0 * PI / 8.0);
                    if (touch_id.1 == 'D' || touch_id.1 == 'E') {
                        touch_angle += PI / 16.0;
                    }
                    let touch_radius = match touch_id.1.to_ascii_uppercase() {
                        'C' => 0.0,
                        'B' => 0.4 * judgment_radius,
                        'E' => 0.5 * judgment_radius,
                        'A' | 'D' => 0.8,
                        _ => 0.0,
                    };
                    let touch_spawn_pos = polar_to_cartesian(touch_radius, touch_angle);
                    let touch_note_transform =
                        Transform::from_translation(touch_spawn_pos.extend(2.0));

                    // D. Convert your parser's NoteDetail into your ECS NoteType component
                    let note_type_component = match note.note_type {
                        NoteDetail::Tap(id) => NoteType::Tap(id),
                        NoteDetail::TapHold(id, (divider, length)) => {
                            NoteType::TapHold(id, (divider, length))
                        }
                        NoteDetail::Slide(id, _, duration) => NoteType::Slide(id, duration),
                        NoteDetail::Touch((id, c)) => NoteType::Touch((id, c)),
                        NoteDetail::TouchHold((id, c), (divider, length)) => {
                            NoteType::TouchHold((id, c), (divider, length))
                        }
                        _ => unimplemented!(),
                    };

                    // E. Spawn the base entity with shared logic components
                    let mut growing_time = 1.5 / (chart.chart_speed * chart.note_speed);
                    let mut entity_cmds = commands.spawn((
                        note_type_component,
                        NoteTiming::Growing(Timer::from_seconds(growing_time, TimerMode::Once)),
                    ));

                    // F. Insert ONLY the specific visual assets based on the note type
                    match note.note_type {
                        NoteDetail::Tap(_) => {
                            entity_cmds.insert((
                                Mesh2d(note_assets.tap_mesh.clone()),
                                MeshMaterial2d(if is_paired {
                                    note_assets.paired_material.clone()
                                } else {
                                    note_assets.tap_material.clone()
                                }),
                                button_note_transform,
                            ));
                        }
                        NoteDetail::TapHold(_, (divider, length)) => {
                            let hold_duration =
                                (length as f32 / divider as f32) * (240.0 / chart.bpm);
                            let move_duration = 2.0 / (chart.chart_speed * chart.note_speed);
                            let speed = (judgment_radius - spawn_radius).abs() / move_duration;
                            let tail_length = speed * hold_duration;

                            entity_cmds.insert((button_note_transform, Visibility::default()));
                            entity_cmds.with_children(|parent| {
                                let initial_len = 0.001;
                                // 1. HEAD (Half-Hexagon facing Forward/Up)
                                parent.spawn((
                                    Mesh2d(note_assets.hold_mesh.clone()),
                                    MeshMaterial2d(if is_paired {
                                        note_assets.paired_material.clone()
                                    } else {
                                        note_assets.hold_material.clone()
                                    }),
                                    HoldNoteElement::Head,
                                ));
                                // BODY (Notice the HoldBodyMarker)
                                parent.spawn((
                                    Mesh2d(note_assets.hold_body_mesh.clone()),
                                    MeshMaterial2d(if is_paired {
                                        note_assets.paired_material.clone()
                                    } else {
                                        note_assets.hold_material.clone()
                                    }),
                                    Transform::from_xyz(0.0, -initial_len / 2.0, 0.0)
                                        .with_scale(Vec3::new(1.0, initial_len, 1.0)),
                                    HoldNoteElement::Body,
                                ));

                                // 3. TAIL (Half-Hexagon facing Backward/Down)
                                parent.spawn((
                                    Mesh2d(note_assets.hold_mesh.clone()),
                                    MeshMaterial2d(if is_paired {
                                        note_assets.paired_material.clone()
                                    } else {
                                        note_assets.hold_material.clone()
                                    }),
                                    Transform::from_xyz(0.0, -initial_len, 0.0)
                                        .with_rotation(Quat::from_rotation_z(PI)),
                                    HoldNoteElement::Tail,
                                ));
                            });
                        }
                        NoteDetail::Slide(btn, shape, duration) => {
                            let points = generate_points(&shape, judgment_radius, spawn_radius);
                            println!("Points of on slide: {:?}", points);
                            let total_length = calculate_total_length(&points);
                            entity_cmds.insert((
                                Mesh2d(note_assets.slide_mesh.clone()),
                                MeshMaterial2d(if is_paired {
                                    note_assets.paired_material.clone()
                                } else {
                                    note_assets.slide_material.clone()
                                }),
                                button_note_transform,
                                Visibility::default(), // <-- ADD THIS SO THE QUERY CAN FIND IT!
                                SlidePath {
                                    waypoints: points,
                                    total_length: total_length,
                                    track_entity: None,
                                },
                            ));
                        }
                        NoteDetail::Touch(_) => {
                            entity_cmds.insert((
                                Mesh2d(note_assets.touch_circle_mesh.clone()),
                                MeshMaterial2d(if is_paired {
                                    note_assets.paired_material.clone()
                                } else {
                                    note_assets.slide_material.clone()
                                }),
                                touch_note_transform,
                                TouchElement::Center,
                                Visibility::Hidden,
                            ));
                            entity_cmds.with_children(|parent| {
                                // Spawn the 4 approaching triangles
                                let start_distance = 10.0;
                                let directions = [Vec2::Y, Vec2::NEG_Y, Vec2::NEG_X, Vec2::X];

                                for dir in directions {
                                    parent.spawn((
                                        Mesh2d(note_assets.touch_triangle_mesh.clone()),
                                        MeshMaterial2d(if is_paired {
                                            note_assets.paired_material.clone()
                                        } else {
                                            note_assets.slide_material.clone()
                                        }),
                                        Transform::from_translation(
                                            (dir * start_distance).extend(0.1),
                                        )
                                        // Rotate triangles to point inward
                                        .with_rotation(
                                            Quat::from_rotation_z(
                                                dir.y.atan2(dir.x) - std::f32::consts::PI / 2.0,
                                            ),
                                        ),
                                        TouchElement::Triangle,
                                    ));
                                }
                            });
                        }
                        _ => { /* Handle Touches here later */ }
                    }
                }
            }
            ChartEvent::Rest => waiting = true,
        }

        // 4. Advance the chart and prepare the timer for the next beat
        if chart.current_index < chart.events.len() {
            chart.current_index += 1;
        }
        if waiting {
            break;
        }
    }
    let step_seconds = (240.0 / (chart.bpm * chart.resolution as f32)) / chart.chart_speed;
    timer.set_duration(std::time::Duration::from_secs_f32(step_seconds));
    timer.reset();
}

fn update_note(
    // Note: Made NoteTiming mutable so we can change its state
    mut query: Query<(
        Entity,
        &mut NoteTiming,
        &NoteType,
        &mut Transform,
        Option<&Children>,
        &mut Visibility, //
        Option<&mut SlidePath>,
    )>,
    mut child_hold_query: Query<
        (&mut Transform, &HoldNoteElement),
        (Without<NoteTiming>, Without<TouchElement>),
    >,
    mut child_touch_query: Query<
        (&mut Transform, &TouchElement),
        (Without<NoteTiming>, Without<HoldNoteElement>),
    >,
    // 2. Added these three queries to manage the track!
    mut arrow_query: Query<(Entity, &SlideArrow, &mut Visibility, &mut Shape), Without<NoteTiming>>,
    children_query: Query<&Children>,
    mut chart: ResMut<ChartPlayback>,
    time: Res<Time>, // I renamed 'timer' to 'time' to avoid confusion with the Timer component
    mut commands: Commands,
    sound_resources: Res<SoundResources>,
    audio: Res<Audio>,
    mut window: Single<&Window>,
    mut note_assets: Res<NoteAssets>,
) {
    if !chart.is_playing {
        return; // Early return keeps the rest of the code clean
    }
    // 1. Calculate the judgment ring radius (same math from your spawn system)
    let size = Vec2::new(window.width(), window.height());
    let judgment_radius = size.min_element() / 2.0 - 40.0;
    let spawn_radius = size.min_element() * 0.12;

    for (entity, mut timing, note_type, mut transform, children, mut visibility, mut slide_path) in
        query.iter_mut()
    {
        // We will store the next state here if a timer finishes
        let mut transition_to = None;

        // Match on a mutable reference to the timing enum
        match &mut *timing {
            NoteTiming::Growing(grow_timer) => {
                // 1. Tick the timer forward
                grow_timer.tick(time.delta());

                // 2. Grow the note using the timer's built-in 0.0 -> 1.0 percentage
                let progress = grow_timer.fraction();
                // 2. Only standard notes scale up during growing. Touch notes do nothing because they are hidden.
                if !matches!(note_type, NoteType::Touch(_) | NoteType::TouchHold(_, _)) {
                    transform.scale = Vec3::splat(progress);
                }

                if grow_timer.just_finished() {
                    // 3. Unhide the Touch note right before it starts moving
                    *visibility = Visibility::Visible;
                    // --- NEW: SPAWN THE INVISIBLE TRACK ---
                    if let Some(ref mut path_data) = slide_path {
                        let track_ent = commands
                            .spawn((Transform::default(), Visibility::default()))
                            .with_children(|parent| {
                                let mut current_dist = 0.0;
                                while current_dist <= path_data.total_length {
                                    let (pos, angle) = get_transform_at_distance(
                                        &path_data.waypoints,
                                        current_dist,
                                    );
                                    parent.spawn((
                                        ShapeBuilder::with(&note_assets.chevron_shape)
                                            .stroke((LIGHT_CYAN.with_alpha(0.0), 10.0))
                                            .build(),
                                        Transform::from_translation(pos.extend(0.5))
                                            .with_rotation(Quat::from_rotation_z(angle)),
                                        SlideArrow {
                                            distance_along_path: current_dist,
                                        },
                                        Visibility::Visible, // Starts completely hidden!
                                    ));
                                    current_dist += 35.0; // Spacing
                                }
                            })
                            .id();

                        path_data.track_entity = Some(track_ent);
                    }
                    // ALL notes (including Touch) now transition to Moving
                    transition_to = Some(NoteTiming::Moving(Timer::from_seconds(
                        2.0 / (chart.chart_speed * chart.note_speed),
                        TimerMode::Once,
                    )));
                }
            }
            NoteTiming::Moving(move_timer) => {
                move_timer.tick(time.delta());
                let progress = move_timer.fraction();

                // --- 1. MOVEMENT LOGIC ---
                if let NoteType::Touch(_) | NoteType::TouchHold(_, _) = note_type {
                    // TOUCH NOTES: Animate triangles inward
                    let start_distance = 10.0; // Ensure this matches the spawn distance!
                    let current_dist = start_distance * (1.0 - progress);

                    if let Some(children) = children {
                        for child in children.iter() {
                            // BUG FIX: Add the `*` before child to dereference it properly
                            if let Ok((mut child_transform, element_type)) =
                                child_touch_query.get_mut(child)
                            {
                                if let TouchElement::Triangle = element_type {
                                    let dir =
                                        child_transform.translation.truncate().normalize_or_zero();
                                    child_transform.translation = (dir * current_dist).extend(0.1);
                                }
                            }
                        }
                    }
                } else {
                    // STANDARD NOTES: Move the parent outward
                    let direction = transform.translation.truncate().normalize_or_zero();
                    let current_radius = spawn_radius + (judgment_radius - spawn_radius) * progress;
                    transform.translation =
                        (direction * current_radius).extend(transform.translation.z);

                    if let NoteType::TapHold(_, (divider, length)) = note_type {
                        let hold_duration =
                            (*length as f32 / *divider as f32) * (240.0 / chart.bpm);
                        let move_duration = 2.0 / (chart.chart_speed * chart.note_speed);
                        let speed = (judgment_radius - spawn_radius).abs() / move_duration;
                        let max_tail_length = speed * hold_duration;

                        let distance_traveled = current_radius - spawn_radius;
                        let current_length = distance_traveled.min(max_tail_length);

                        if let Some(children) = children {
                            for child in children.iter() {
                                if let Ok((mut child_transform, element_type)) =
                                    child_hold_query.get_mut(child)
                                {
                                    match element_type {
                                        HoldNoteElement::Body => {
                                            child_transform.scale.y = current_length;
                                            child_transform.translation.y = -current_length / 2.0;
                                        }
                                        HoldNoteElement::Tail => {
                                            child_transform.translation.y = -current_length;
                                        }
                                        HoldNoteElement::Head => {}
                                    }
                                }
                            }
                        }
                    }
                    if let Some(ref path_data) = slide_path {
                        if let Some(track_ent) = path_data.track_entity {
                            if let Ok(track_children) = children_query.get(track_ent) {
                                for child in track_children.iter() {
                                    if let Ok((_, _, _, mut shape)) = arrow_query.get_mut(child) {
                                        if let Some(stroke) = &mut shape.stroke {
                                            stroke.color = LIGHT_CYAN.with_alpha(progress).into();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // --- 2. DESPAWN LOGIC ---
                // BUG FIX: This must be OUTSIDE the `else` block so Touch notes can reach it!
                if move_timer.just_finished() {
                    audio.play(sound_resources.hit.clone());
                    match note_type {
                        NoteType::TapHold(_, (divider, length)) => {
                            let hold_duration =
                                (*length as f32 / *divider as f32) * (240.0 / chart.bpm);
                            let move_duration = 2.0 / (chart.chart_speed * chart.note_speed);
                            let speed = (judgment_radius - spawn_radius).abs() / move_duration;
                            let max_tail_length = speed * hold_duration;

                            transition_to = Some(NoteTiming::Holding(
                                Timer::from_seconds(hold_duration, TimerMode::Once),
                                max_tail_length,
                            ));
                        }
                        // Add TouchHold transition here later when you are ready!
                        NoteType::Slide(_, duration) => {
                            commands
                                .entity(entity)
                                .insert(Mesh2d(note_assets.slide_mesh.clone()));
                            commands
                                .entity(entity)
                                .insert(MeshMaterial2d(note_assets.slide_material.clone()));
                            // Wait 1 Beat before sliding
                            let one_beat_duration = 60.0 / chart.bpm;
                            transition_to = Some(NoteTiming::Waiting(Timer::from_seconds(
                                one_beat_duration,
                                TimerMode::Once,
                            )));
                        }
                        _ => {
                            commands.entity(entity).despawn();
                        }
                    }
                }
            }
            NoteTiming::Holding(hold_timer, max_tail_length) => {
                hold_timer.tick(time.delta());

                // --- NEW: DYNAMICALLY SHRINK THE BODY ---
                // Recalculate speed so we shrink at the exact correct velocity
                let move_duration = 2.0 / (chart.chart_speed * chart.note_speed);
                let speed = (judgment_radius - spawn_radius).abs() / move_duration;

                // The length of the tail is exactly the time remaining * speed
                let time_remaining = hold_timer.remaining_secs();
                let expected_length = time_remaining * speed;

                // It is clamped so the tail never crosses backwards past the center spawn ring!
                let current_length = expected_length.min(judgment_radius - spawn_radius);

                if let Some(children) = children {
                    for child in children.iter() {
                        if let Ok((mut child_transform, element_type)) =
                            child_hold_query.get_mut(child)
                        {
                            match element_type {
                                HoldNoteElement::Body => {
                                    child_transform.scale.y = current_length;
                                    child_transform.translation.y = -current_length / 2.0;
                                }
                                HoldNoteElement::Tail => {
                                    child_transform.translation.y = -current_length;
                                }
                                HoldNoteElement::Head => {}
                            }
                        }
                    }
                }

                if hold_timer.just_finished() {
                    audio.play(sound_resources.hit.clone());
                    commands.entity(entity).despawn();
                }
            }
            // --- THE NEW SLIDE PHASES ---
            NoteTiming::Waiting(wait_timer) => {
                wait_timer.tick(time.delta());
                if wait_timer.just_finished() {
                    if let NoteType::Slide(_, (divider, length)) = note_type {
                        let slide_duration =
                            (*length as f32 / *divider as f32) * (240.0 / chart.bpm);
                        transition_to = Some(NoteTiming::Sliding(Timer::from_seconds(
                            slide_duration,
                            TimerMode::Once,
                        )));
                    }
                }
            }
            NoteTiming::Sliding(slide_timer) => {
                slide_timer.tick(time.delta());
                let progress = slide_timer.fraction();

                if let Some(ref path_data) = slide_path {
                    // 1. Move the Star
                    let current_distance = progress * path_data.total_length;
                    let (new_pos, _) =
                        get_transform_at_distance(&path_data.waypoints, current_distance);
                    transform.translation = new_pos.extend(transform.translation.z);

                    // 2. Eat the Track
                    if let Some(track_ent) = path_data.track_entity {
                        if let Ok(track_children) = children_query.get(track_ent) {
                            for child in track_children.iter() {
                                if let Ok((arrow_ent, arrow, _, _)) = arrow_query.get_mut(child) {
                                    if arrow.distance_along_path <= current_distance {
                                        commands.entity(arrow_ent).despawn();
                                    }
                                }
                            }
                        }
                    }

                    // 3. Clean up
                    if slide_timer.just_finished() {
                        commands.entity(entity).despawn();
                        if let Some(track_ent) = path_data.track_entity {
                            commands.entity(track_ent).despawn();
                        }
                    }
                }
            }
        }

        // If a transition was queued up, apply it now
        if let Some(new_state) = transition_to {
            *timing = new_state;
        }
    }
}
