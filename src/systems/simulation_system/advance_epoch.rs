use crate::components::{Coord, Organism};
use crate::resources::*;
use crate::systems::*;
use bevy::prelude::*;
use rand::Rng;

pub fn advance_epoch(
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

        //organisms death
        for (e, mut org, coord) in orgs_query.iter_mut() {
            org.age += 1;

            if org.age >= params.average_lifespan
                && rng.gen_bool(
                    (0.5 + (org.age as f64 / params.average_lifespan as f64)).clamp(0.5, 1.),
                )
            {
                grid.set(coord.x as usize, coord.y as usize, CellType::Consumable);
                spawn_pellet(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    coord,
                    params.cell_width,
                    params.cell_height,
                );

                commands.entity(e).despawn();

                n_entities -= 1;
                species.decrement_species(org.species);

                continue;
            } else {
                total_orgs_energy += org.energy;
            }
        }

        species.remove_dead_species();

        if n_entities == 0 {
            return;
        }

        let pellet_coords = generate_pellets(total_orgs_energy, &grid);
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
