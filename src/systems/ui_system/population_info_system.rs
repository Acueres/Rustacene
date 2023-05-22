use crate::components::{ui::*, Organism};
use bevy::prelude::*;

pub fn population_info_system(
    orgs_query: Query<&Organism>,
    mut population_ui_query: Query<&mut Text, With<PopulationText>>,
) {
    let mut population_text = population_ui_query.get_single_mut().unwrap();
    population_text.sections[1].value = orgs_query.iter().len().to_string();
}
