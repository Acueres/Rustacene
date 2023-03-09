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
    mut orgs_query: Query<(Entity, &mut Organism, &Coord<isize>)>,
) {
    if !sim_state.paused && !sim_state.reset && epoch_time.timer.tick(time.delta()).just_finished()
    {
        let mut rng = rand::thread_rng();
        let mut children = Vec::<(Organism, Coord<isize>)>::new();

        sim_state.epoch += 1;

        let mut n_entities = orgs_query.iter().len();

        for (e, mut org, coord) in orgs_query.iter_mut() {
            org.age += 1;

            if org.energy.is_sign_negative()
                || (org.age >= params.average_lifespan
                    && rng.gen_bool(
                        (0.5 + (org.age as f64 / params.average_lifespan as f64)).clamp(0., 1.),
                    ))
            {
                grid.set(coord.x as usize, coord.y as usize, CellType::Empty);
                commands.entity(e).despawn_recursive();
                n_entities -= 1;
                continue;
            }

            if n_entities >= params.n_max_entities || org.energy <= 0.2 {
                continue;
            }

            let nearby_coords = grid
                .clone()
                .search_area(coord.to_owned(), 1, CellType::Empty);
            if nearby_coords.len() > 0 {
                let child_coord = nearby_coords[rng.gen_range(0..nearby_coords.len())];
                let child = org
                    .clone()
                    .replicate(0.05, params.genome_len, params.ns_shape);
                org.energy -= 0.2;
                children.push((child, child_coord));
                grid.set(
                    child_coord.x as usize,
                    child_coord.y as usize,
                    CellType::Impassable,
                );
                n_entities += 1;
            }
        }

        for (org, coord) in children.iter() {
            spawn_organism(
                &mut commands,
                &mut meshes,
                &mut materials,
                org,
                coord,
                params.cell_width,
                params.cell_height,
            );
        }

        if n_entities == 0 {
            return;
        }

        let pellet_coords = generate_pellets(n_entities, &grid);
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
