use bevy::prelude::{Resource, Timer};

#[derive(Resource)]
pub struct SimTime {
    pub timer: Timer,
}
