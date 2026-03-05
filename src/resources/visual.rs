use bevy::prelude::*;
use bevy_prototype_lyon::prelude::ShapePath;

#[derive(Resource)]
pub struct NoteAssets {
    pub tap_mesh: Handle<Mesh>,
    pub hold_mesh: Handle<Mesh>,
    pub hold_body_mesh: Handle<Mesh>,
    pub slide_mesh: Handle<Mesh>,
    pub touch_circle_mesh: Handle<Mesh>, 
    pub touch_triangle_mesh: Handle<Mesh>,
    pub chevron_shape: ShapePath, 
    pub tap_material: Handle<ColorMaterial>,
    pub hold_material: Handle<ColorMaterial>,
    pub slide_material: Handle<ColorMaterial>,
    pub paired_material: Handle<ColorMaterial>,
}
