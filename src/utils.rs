use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::Vec2;
use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::{PI, FRAC_PI_2};

use crate::parser::element::SlideShape;

pub fn random_number(min: f32, max: f32) -> f32 {
    let mut rng = rand::rng();
    return rng.random_range(min..max);
}

// Helper function if you don't already have it
pub fn polar_to_cartesian(radius: f32, angle: f32) -> Vec2 {
    Vec2::new(radius * angle.cos(), radius * angle.sin())
}

// This takes your two shapes and perfectly stitches them into a hollow arch!
pub fn create_hollow_arch(radius: f32) -> Mesh {
    // 1. Calculate the exact points based on your math
    let r_in = radius * 0.75;
    let y_out = radius * 0.5;
    let y_in = y_out * 0.75;

    // Ordered Left -> Peak -> Right
    let positions = vec![
        // Outer edge (indices 0 to 4)
        [-radius, 0.0, 0.0],
        [-radius, y_out, 0.0],
        [0.0, radius, 0.0],
        [radius, y_out, 0.0],
        [radius, 0.0, 0.0],
        // Inner edge (indices 5 to 9)
        [-r_in, 0.0, 0.0],
        [-r_in, y_in, 0.0],
        [0.0, r_in, 0.0],
        [r_in, y_in, 0.0],
        [r_in, 0.0, 0.0],
    ];

    let normals = vec![[0.0, 0.0, 1.0]; 10];
    let uvs = vec![[0.0, 0.0]; 10];

    // Stitch the 4 segments together, leaving the bottom edge completely open!
    let mut indices = Vec::new();
    for i in 0..4 {
        indices.extend_from_slice(&[i, i + 5, i + 6, i, i + 6, i + 1]);
    }

    // Bevy 0.18 compliant mesh initialization
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Generates a procedural hollow star mesh where the inner shape perfectly matches the outer
pub fn create_hollow_star_mesh(radius: f32, ratio: f32, thickness: f32, num_points: usize) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let num_vertices_per_ring = num_points * 2;

    // Outer ring dimensions
    let r_outer_tip = radius;
    let r_outer_dip = radius * ratio;

    // Inner ring dimensions (Maintains the exact same shape/proportions)
    let r_inner_tip = r_outer_tip - thickness;
    let r_inner_dip = r_inner_tip * ratio; // FIX: Use the same ratio as the outer ring!

    // Generate Outer Ring
    for i in 0..num_vertices_per_ring {
        let angle = (PI / 2.0) + (i as f32 * PI / num_points as f32);
        let current_radius = if i % 2 == 0 { r_outer_tip } else { r_outer_dip };

        let x = angle.cos() * current_radius;
        let y = angle.sin() * current_radius;

        positions.push([x, y, 0.0]);
        normals.push([0.0, 0.0, 1.0]);

        let norm_x = x / r_outer_tip;
        let norm_y = y / r_outer_tip;
        uvs.push([(norm_x + 1.0) / 2.0, (1.0 - norm_y) / 2.0]);
    }

    // Generate Inner Ring
    for i in 0..num_vertices_per_ring {
        let angle = (PI / 2.0) + (i as f32 * PI / num_points as f32);
        let current_radius = if i % 2 == 0 { r_inner_tip } else { r_inner_dip };

        let x = angle.cos() * current_radius;
        let y = angle.sin() * current_radius;

        positions.push([x, y, 0.0]);
        normals.push([0.0, 0.0, 1.0]);

        let norm_x = x / r_outer_tip;
        let norm_y = y / r_outer_tip;
        uvs.push([(norm_x + 1.0) / 2.0, (1.0 - norm_y) / 2.0]);
    }

    // Generate Triangles
    for i in 0..num_vertices_per_ring {
        let current_outer = i as u32;
        let next_outer = ((i + 1) % num_vertices_per_ring) as u32;
        let current_inner = (i + num_vertices_per_ring) as u32;
        let next_inner = (((i + 1) % num_vertices_per_ring) + num_vertices_per_ring) as u32;

        indices.push(current_outer);
        indices.push(next_outer);
        indices.push(current_inner);

        indices.push(next_outer);
        indices.push(next_inner);
        indices.push(current_inner);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

// 1. Calculate the total physical distance of the path
pub fn calculate_total_length(points: &[Vec2]) -> f32 {
    let mut length = 0.0;
    for window in points.windows(2) {
        length += window[0].distance(window[1]);
    }
    length
}

// 2. Find the exact Position and Angle at a specific distance along the path
pub fn get_transform_at_distance(points: &[Vec2], target_distance: f32) -> (Vec2, f32) {
    let mut current_dist = 0.0;

    for window in points.windows(2) {
        let p1 = window[0];
        let p2 = window[1];
        let segment_length = p1.distance(p2);

        // If the target distance falls inside this specific line segment:
        if current_dist + segment_length >= target_distance {
            // Find how far along this specific segment we are (0.0 to 1.0)
            let t = (target_distance - current_dist) / segment_length;

            // Interpolate the exact position
            let exact_pos = p1.lerp(p2, t);

            // Calculate the angle of this segment
            let angle = (p2.y - p1.y).atan2(p2.x - p1.x);

            return (exact_pos, angle);
        }
        current_dist += segment_length;
    }

    // Fallback: If we ask for a distance past the end, return the very last point
    let last = points.last().unwrap();
    let prev = points[points.len() - 2];
    let angle = (last.y - prev.y).atan2(last.x - prev.x);
    (*last, angle)
}

pub fn generate_points(shape: &SlideShape, boundary_radius: f32) -> Vec<Vec2> {
    let spacing = 35.0; // Desired spacing between generated points

    // Helper closure to calculate the angle from a button index (assumes 1-8 indexing)
    let button_angle = |button: usize| -> f32 {
        FRAC_PI_2 - (-0.5 + button as f32) * (PI / 4.0)
    };

    // Helper closure to calculate the XY position from a button index
    let button_pos = |button: usize| -> Vec2 {
        let a = button_angle(button);
        Vec2::new(boundary_radius * a.cos(), boundary_radius * a.sin())
    };

    match shape {
        SlideShape::Straight(start, end) => {
            let start_pos = button_pos(*start);
            let end_pos = button_pos(*end);

            let distance = start_pos.distance(end_pos);
            let steps = (distance / spacing).ceil().max(1.0) as usize;
            let mut points = Vec::with_capacity(steps + 1);

            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                points.push(start_pos.lerp(end_pos, t));
            }
            points
        }

        SlideShape::ShortArc(start, end) => {
            let start_ang = button_angle(*start);
            let mut end_ang = button_angle(*end);

            // Calculate the shortest path (-PI to PI)
            let mut diff = (end_ang - start_ang) % (2.0 * PI);
            if diff > PI { diff -= 2.0 * PI; }
            if diff < -PI { diff += 2.0 * PI; }
            end_ang = start_ang + diff;

            generate_arc_points(start_ang, end_ang, boundary_radius, spacing)
        }

        SlideShape::ClockwiseArc(start, end) => {
            let start_ang = button_angle(*start);
            let mut end_ang = button_angle(*end);

            // Clockwise means angle decreases
            while end_ang > start_ang { 
                end_ang -= 2.0 * PI; 
            }

            generate_arc_points(start_ang, end_ang, boundary_radius, spacing)
        }

        SlideShape::CounterClockwiseArc(start, end) => {
            let start_ang = button_angle(*start);
            let mut end_ang = button_angle(*end);

            // Counter-clockwise means angle increases
            while end_ang < start_ang { 
                end_ang += 2.0 * PI; 
            }

            generate_arc_points(start_ang, end_ang, boundary_radius, spacing)
        }

        SlideShape::VShape(start, end) => {
            let start_pos = button_pos(*start);
            let center_pos = Vec2::ZERO;
            let end_pos = button_pos(*end);

            generate_multi_segment_points(&[start_pos, center_pos, end_pos], spacing)
        }

        SlideShape::GrandVShape(start, end, mid) => {
            let start_pos = button_pos(*start);
            let mid_pos = button_pos(*mid);
            let end_pos = button_pos(*end);

            // Note: Passed in order of Start -> Mid -> End
            generate_multi_segment_points(&[start_pos, mid_pos, end_pos], spacing)
        }

        SlideShape::PShape(_start, _end) => unimplemented!("PShape (p) requires Bezier curve implementation"),
        SlideShape::QShape(_start, _end) => unimplemented!("QShape (q) requires Bezier curve implementation"),
        SlideShape::GrandPShape(_start, _end) => unimplemented!("GrandPShape (pp) requires Bezier curve implementation"),
        SlideShape::GrandQShape(_start, _end) => unimplemented!("GrandQShape (qq) requires Bezier curve implementation"),
        SlideShape::Thunderbolt(_start, _end) => unimplemented!("Thunderbolt (s/z) requires zig-zag offset logic"),
        
        // Note: Fan shapes spawn 3 stars! You might need to change your engine architecture 
        // to return `Vec<Vec<Vec2>>` so you can handle all 3 paths at once.
        SlideShape::FanShape(_start, (_end1, _end2, _end3)) => unimplemented!("FanShape (w) requires splitting into 3 paths"),
    }
}

// --- Helper Functions to keep the match statement clean ---

fn generate_arc_points(start_angle: f32, end_angle: f32, radius: f32, spacing: f32) -> Vec<Vec2> {
    let angle_diff = (end_angle - start_angle).abs();
    let arc_length = radius * angle_diff;
    let steps = (arc_length / spacing).ceil().max(1.0) as usize;
    let mut points = Vec::with_capacity(steps + 1);

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let current_angle = start_angle + (end_angle - start_angle) * t;
        points.push(Vec2::new(
            radius * current_angle.cos(),
            radius * current_angle.sin(),
        ));
    }
    points
}

fn generate_multi_segment_points(waypoints: &[Vec2], spacing: f32) -> Vec<Vec2> {
    let mut points = Vec::new();
    
    for w in 0..(waypoints.len() - 1) {
        let p1 = waypoints[w];
        let p2 = waypoints[w + 1];
        let distance = p1.distance(p2);
        let steps = (distance / spacing).ceil().max(1.0) as usize;
        
        // To avoid duplicating the exact waypoint where segments connect, 
        // skip the first point of the next segment if it's not the very first segment.
        let start_i = if w == 0 { 0 } else { 1 };

        for i in start_i..=steps {
            let t = i as f32 / steps as f32;
            points.push(p1.lerp(p2, t));
        }
    }
    points
}
