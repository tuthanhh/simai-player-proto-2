use bevy::prelude::*;

mod bevy_enhanced_input;
mod bevy_kira_audio;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((bevy_enhanced_input::plugin, bevy_kira_audio::plugin));
}
