use bevy::prelude::Resource;

#[derive(Resource)]
pub struct SimState {
    pub paused: bool,
    pub reset: bool,
    pub epoch: usize,
}
