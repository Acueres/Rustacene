use bevy::prelude::*;

mod components;
mod resources;
mod sim;
mod systems;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(sim::SimPlugin)
        .run();
}
