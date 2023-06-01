use crate::components::{Coord, Organism};
use crate::resources::*;
use crate::systems::*;
use bevy::prelude::*;
use rand::Rng;

pub fn epoch_system(
    mut commands: Commands,
    time: Res<Time>,
    mut sim_state: ResMut<SimState>,
    params: Res<Parameters>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut epoch_time: ResMut<EpochTime>,
    mut grid: ResMut<Grid>,
    mut species: ResMut<Species>,
    mut orgs_query: Query<(Entity, &mut Organism, &Coord<isize>)>,
) {
    if !sim_state.paused && !sim_state.reset && epoch_time.timer.tick(time.delta()).just_finished()
    {
        let mut rng = rand::thread_rng();
        let mut total_orgs_energy: f32 = 0.;

        sim_state.epoch += 1;

        let mut n_entities = orgs_query.iter().len();

        //organism death
        for (e, mut org, coord) in orgs_query.iter_mut() {
            org.age += 1;

            if org.age > params.average_lifespan
                && rng
                    .gen_bool((org.age as f64 / params.average_lifespan as f64 - 1.).clamp(0., 1.))
            {
                n_entities -= 1;
                grid.set(coord.x as usize, coord.y as usize, CellType::Empty);
                species.decrement_species(org.species);

                commands.entity(e).despawn_recursive();
                continue;
            } else {
                total_orgs_energy += org.energy;
            }
        }

        if n_entities == 0 {
            return;
        }

        let pellet_coords = energy_system(total_orgs_energy, &grid);
        for coord in pellet_coords.iter() {
            grid.set(coord.x as usize, coord.y as usize, CellType::Consumable);
            spawn_pellet(
                &mut commands,
                &mut meshes,
                &mut materials,
                coord,
                params.cell_width,
                params.cell_height,
            );
        }
    }
}
