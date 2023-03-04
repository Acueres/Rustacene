use crate::resources::*;
use bevy::prelude::*;
use bevy::utils::Duration;

pub fn handle_input(
    keys: Res<Input<KeyCode>>,
    mut sim_time: ResMut<SimTime>,
    mut epoch_time: ResMut<EpochTime>,
    mut sim_state: ResMut<SimState>,
) {
    if keys.just_pressed(KeyCode::Space) {
        sim_state.paused ^= true;
    }
    if keys.just_pressed(KeyCode::R) {
        sim_state.reset = true;
    }

    //sim speed control
    if keys.just_pressed(KeyCode::Key1) {
        sim_time.timer.set_duration(Duration::from_secs_f32(0.1));
        epoch_time.timer.set_duration(Duration::from_secs_f32(10.));
    }
    if keys.just_pressed(KeyCode::Key2) {
        sim_time.timer.set_duration(Duration::from_secs_f32(0.05));
        epoch_time.timer.set_duration(Duration::from_secs_f32(5.));
    }
    if keys.just_pressed(KeyCode::Key3) {
        sim_time.timer.set_duration(Duration::from_secs_f32(0.025));
        epoch_time.timer.set_duration(Duration::from_secs_f32(2.5));
    }
}
