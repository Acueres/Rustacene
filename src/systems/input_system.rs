use crate::resources::*;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use std::time::Duration;

pub fn input_system(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut scroll_events: EventReader<MouseWheel>,
    time: Res<Time>,
    mut sim_time: ResMut<SimTime>,
    mut epoch_time: ResMut<EpochTime>,
    mut sim_state: ResMut<SimState>,
) {
    if keys.just_pressed(KeyCode::Space) {
        sim_state.paused ^= true;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        sim_state.reset = true;
    }

    //sim speed control
    if keys.just_pressed(KeyCode::Digit1) {
        sim_time.timer.set_duration(Duration::from_secs_f32(0.1));
        epoch_time.timer.set_duration(Duration::from_secs_f32(10.));
    }
    if keys.just_pressed(KeyCode::Digit2) {
        sim_time.timer.set_duration(Duration::from_secs_f32(0.05));
        epoch_time.timer.set_duration(Duration::from_secs_f32(5.));
    }
    if keys.just_pressed(KeyCode::Digit3) {
        sim_time.timer.set_duration(Duration::from_secs_f32(0.025));
        epoch_time.timer.set_duration(Duration::from_secs_f32(2.5));
    }

    //camera zoom
    let mut camera = camera_query.single_mut().unwrap();
    for ev in scroll_events.read() {
        let mut log_scale = Vec3 {
            x: camera.scale.x.ln(),
            y: camera.scale.y.ln(),
            z: camera.scale.z.ln(),
        };
        log_scale -= 5. * ev.y.signum() * time.delta_secs();
        camera.scale = log_scale.exp();
        camera.scale = camera.scale.clamp(
            Vec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            Vec3 {
                x: 1.,
                y: 1.,
                z: 1.,
            },
        );
    }

    let scale_factor = 1.2 - camera.scale.x;
    let delta = 5e2 * scale_factor * time.delta_secs();
    let mut camera_delta_y = 0.;
    let mut camera_delta_x = 0.;

    if keys.pressed(KeyCode::KeyW) {
        camera_delta_y += delta;
    }
    if keys.pressed(KeyCode::KeyS) {
        camera_delta_y -= delta;
    }
    if keys.pressed(KeyCode::KeyD) {
        camera_delta_x += delta;
    }
    if keys.pressed(KeyCode::KeyA) {
        camera_delta_x -= delta;
    }

    camera.translation.x += camera_delta_x;
    camera.translation.y += camera_delta_y;
}
