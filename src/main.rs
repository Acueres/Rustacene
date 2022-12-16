use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod organism;
mod coord;
mod dir;
mod grid;
mod sim;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(sim::SimPlugin)
        .run();
}
