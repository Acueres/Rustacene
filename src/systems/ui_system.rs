use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;

pub fn sim_info(sim_state: Res<SimState>, orgs_query: Query<&Organism>, grid: Res<Grid>) {
    let total_org_energy = orgs_query.iter().map(|org| org.energy).sum::<f32>();
    let total_pellet_energy =
        grid.get_cell_coords(CellType::Consumable).iter().len() as f32 * PELLET_ENERGY;
    let total_system_energy: f32 = total_org_energy + total_pellet_energy;
    print!("{}\r", (0..100).map(|_| " ").collect::<String>());
    print!(
        "Sim info: epoch {}, n_entities {}, total_energy: {}\r",
        sim_state.epoch,
        orgs_query.iter().len(),
        total_system_energy
    );
}
