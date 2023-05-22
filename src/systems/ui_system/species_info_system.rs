use crate::components::ui::*;
use crate::resources::Species;
use bevy::prelude::*;

pub fn species_info_system(
    species: Res<Species>,
    mut total_species_ui_query: Query<&mut Text, (With<TotalSpeciesText>, Without<SpeciesText>)>,
    mut species_ui_query: Query<&mut Text, With<SpeciesText>>,
) {
    let top_species = species.topk(5);

    let mut total_species_text = total_species_ui_query.get_single_mut().unwrap();
    total_species_text.sections[1].value = species.len().to_string();

    for (i, mut text) in species_ui_query.iter_mut().enumerate() {
        let species_index = top_species[i].0;
        text.sections[1].value = species_index.to_string();
        text.sections[1].style.color = species.get_color(species_index);

        text.sections[3].value = top_species[i].1.to_string();
    }
}
