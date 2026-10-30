use crate::components::ui::*;
use crate::resources::*;
use bevy::prelude::*;

pub fn epoch_info_system(
    sim_state: Res<SimState>,
    mut epoch_text: Single<&mut Text, With<EpochText>>,
) {
    epoch_text.set_if_neq(Text::new(format!("Epoch {}", sim_state.epoch + 1)));
}
