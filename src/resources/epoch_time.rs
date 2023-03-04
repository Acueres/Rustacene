use bevy::prelude::{Resource, Timer};

#[derive(Resource)]
pub struct EpochTime {
    pub timer: Timer,
}
