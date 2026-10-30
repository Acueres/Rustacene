use crate::components::{Organism, ui::*};
use bevy::prelude::*;

pub fn population_info_system(
    orgs_query: Query<&Organism>,
    mut population_text: Single<&mut Text, With<PopulationText>>,
) {
    population_text.set_if_neq(Text::new(format!("Population {}", orgs_query.iter().len())));
}
