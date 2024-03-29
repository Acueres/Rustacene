use crate::components::ui::*;
use crate::resources::*;
use bevy::prelude::*;

pub fn epoch_info_system(
    sim_state: Res<SimState>,
    mut epoch_ui_query: Query<&mut Text, With<EpochText>>,
) {
    let mut epoch_text = epoch_ui_query.get_single_mut().unwrap();
    epoch_text.sections[1].value = (sim_state.epoch + 1).to_string();
}
