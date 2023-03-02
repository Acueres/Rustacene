use crate::models::*;
use crate::systems::*;
use bevy::prelude::*;

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimTime {
            timer: Timer::from_seconds(0.05, TimerMode::Repeating),
        })
        .insert_resource(EpochTime {
            timer: Timer::from_seconds(5., TimerMode::Repeating),
        })
        .insert_resource(SimState {
            paused: false,
            reset: false,
            epoch: 0,
        })
        .add_startup_system(setup_sim)
        .add_system(handle_input)
        .add_system(execute_actions)
        .add_system(advance_epoch)
        .add_system(reset_world)
        .add_system(center_camera)
        .add_system(ui);
    }
}
