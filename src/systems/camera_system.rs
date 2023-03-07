use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub fn center_camera(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    let window = window_query.get_single().unwrap();
    let mut camera = camera_query.get_single_mut().unwrap();

    camera.translation.x = window.width() / 2.;
    camera.translation.y = window.height() / 2.;
}
