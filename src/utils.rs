use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::Vec2;
use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

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
