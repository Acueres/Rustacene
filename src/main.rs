use bevy::prelude::*;

mod components;
mod resources;
mod sim;
mod systems;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rustacene".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugin(sim::SimPlugin)
        .run();
}
