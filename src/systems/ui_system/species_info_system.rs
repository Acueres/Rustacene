use crate::components::ui::*;
use crate::resources::Species;
use bevy::prelude::*;

pub fn species_info_system(
    species: Res<Species>,
    mut total_species_text: Single<&mut Text, With<TotalSpeciesText>>,
    mut species_ui_query: Query<(&mut Text, &mut TextColor), (With<SpeciesText>, Without<TotalSpeciesText>)>,
) {
    let top_species = species.topk(5);

    total_species_text.0 = "Total_species: ".to_owned() + &species.len().to_string();

    for (i, (mut text, mut color)) in species_ui_query.iter_mut().enumerate() {
        let species_index = top_species[i].0;
        text.0 = "Species ".to_owned() + &species_index.to_string() + " : " + &top_species[i].1.to_string();
        *color = TextColor(species.get_color(species_index).into());
    }
}
