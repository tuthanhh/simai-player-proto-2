#![allow(unused_imports)]

use bevy::prelude::*;

mod components;
mod parser;
mod plugins;
mod resources;
mod styles;
mod third_party;
mod utils;
pub mod cli;

/// Use this module instead of importing the `components`, `plugins`, `resources`, and `utils`
/// modules directly.
mod prelude {
    pub use super::*;
    pub use {cli::*, components::*, plugins::*, resources::*, styles::*, utils::*};
}
use bevy_kira_audio::prelude::*;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            plugins::camera::plugin,
            plugins::defaults::plugin,
            plugins::fonts::plugin,
            plugins::game::plugin,
            // plugins::input::plugin,
            // plugins::physics::plugin,
            third_party::plugin,
        ));

        // Enable dev tools for dev builds.
        #[cfg(feature = "dev")]
        app.add_plugins(plugins::debug::plugin);
    }
}
