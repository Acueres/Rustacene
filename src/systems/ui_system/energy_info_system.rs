use crate::components::ui::*;
use crate::components::{CellType, Organism, PELLET_ENERGY};
use crate::resources::*;
use bevy::prelude::*;

pub fn energy_info_system(
    grid: Res<Grid>,
    orgs_query: Query<&Organism>,
    mut energy_text: Single<&mut Text, With<EnergyText>>,
) {
    let total_org_energy = orgs_query.iter().map(|org| org.energy).sum::<f32>();
    let total_pellet_energy =
        grid.get_cell_coords(CellType::Consumable).iter().len() as f32 * PELLET_ENERGY;
    let total_system_energy: f32 = total_org_energy + total_pellet_energy;

    energy_text.0 = ((total_system_energy * 100.).round() / 100.).to_string();
}
