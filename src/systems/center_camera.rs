use bevy::prelude::*;

pub fn center_camera(
    mut windows: ResMut<Windows>,
    mut camera: Query<&mut Transform, With<Camera>>,
) {
    for window in windows.iter_mut() {
        for mut transform in camera.iter_mut() {
            transform.translation.x = window.width() / 2.;
            transform.translation.y = window.height() / 2.;
        }
    }
}
