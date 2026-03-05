use bevy::prelude::*;

use simai_player::AppPlugin;
fn main() {
    App::new()
        .add_plugins(AppPlugin)
        .run();
}
