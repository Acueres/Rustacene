use bevy::prelude::*;

mod sim;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(sim::SimPlugin)
        .run();
}
