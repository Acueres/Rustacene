use crate::resources::*;
use crate::systems::*;
use bevy::prelude::*;

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimTime {
            timer: Timer::from_seconds(0.05, TimerMode::Repeating),
        })
        .insert_resource(EpochTime {
            timer: Timer::from_seconds(10., TimerMode::Repeating),
        })
        .insert_resource(SimState {
            paused: false,
            reset: false,
            epoch: 0,
        })
        .add_systems(Startup, (sim_startup_system, ui_startup_system))
        .add_systems(Update, (input_system, sim_step_system, epoch_system, reset_system))
        .add_systems(PostUpdate, (
            energy_info_system,
            epoch_info_system,
            population_info_system,
            species_info_system,
        ));
    }
}
