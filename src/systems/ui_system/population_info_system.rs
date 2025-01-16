use crate::components::{ui::*, Organism};
use bevy::prelude::*;

pub fn population_info_system(
    orgs_query: Query<&Organism>,
    mut population_text: Single<&mut Text, With<PopulationText>>,
) {
    population_text.0 = "Population ".to_owned() + &orgs_query.iter().len().to_string();
}
