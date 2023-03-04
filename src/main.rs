use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod models;
mod resources;
mod sim;
mod systems;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(sim::SimPlugin)
        .run();
}
