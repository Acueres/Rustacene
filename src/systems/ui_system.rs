use crate::components::Organism;
use crate::resources::*;
use bevy::prelude::*;

pub fn sim_info(sim_state: Res<SimState>, orgs_query: Query<&Organism>) {
    print!("{}\r", (0..100).map(|_| " ").collect::<String>());
    print!(
        "Sim info: epoch {}, n_entities {}\r",
        sim_state.epoch,
        orgs_query.iter().len()
    );
}
