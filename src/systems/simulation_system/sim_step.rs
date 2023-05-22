use super::*;
use crate::components::{Action, Coord, Organism};
use crate::resources::*;
use crate::systems::*;
use bevy::prelude::*;
use rand::Rng;

pub fn sim_step(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    sim_state: Res<SimState>,
    params: Res<Parameters>,
    mut sim_time: ResMut<SimTime>,
    mut grid: ResMut<Grid>,
    mut species: ResMut<Species>,
    mut orgs_query: Query<(
        Entity,
        &mut Organism,
        &mut NeuralSystem,
        &mut Coord<isize>,
        &mut Dir,
        &mut Transform,
    )>,
    pellets_query: Query<(Entity, &Coord<isize>, With<Pellet>, Without<Organism>)>,
) {
    if !sim_state.paused && !sim_state.reset && sim_time.timer.tick(time.delta()).just_finished() {
        let mut rng = rand::thread_rng();
        let mut children = Vec::<(Organism, Coord<isize>)>::new();
        let mut pellets_to_remove = Vec::<Coord<isize>>::new();

        for (e, mut org, mut ns, mut coord, mut curr_dir, mut transform) in orgs_query.iter_mut() {
            if org.energy.is_sign_negative() {
                grid.set(coord.x as usize, coord.y as usize, CellType::Consumable);
                spawn_pellet(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &coord,
                    params.cell_width,
                    params.cell_height,
                );

                commands.entity(e).despawn();
                continue;
            }

            let inputs = read_sensors(
                &org,
                &coord.to_owned(),
                &grid.to_owned(),
                &params.to_owned(),
                *curr_dir,
            );

            let action = ns.get_action(inputs);
            org.sub_energy(NS_ENERGY_COST); // thinking requires energy

            if action == Action::Halt {
                continue;
            }

            let dir = action.get_dir(*curr_dir);
            *curr_dir = dir;
            let dir_coord: Coord<isize> = dir.value();
            let next_coord = coord.to_owned() + dir_coord;

            //world bounds check
            if next_coord.x < 0
                || next_coord.x >= params.grid_size as isize
                || next_coord.y < 0
                || next_coord.y >= params.grid_size as isize
            {
                continue;
            }

            //collision check
            if grid.get(next_coord.x as usize, next_coord.y as usize) == CellType::Impassable {
                continue;
            } else if grid.get(next_coord.x as usize, next_coord.y as usize) == CellType::Consumable
            {
                org.add_energy(PELLET_ENERGY); // consuming pellet gives energy
                org.energy = org.energy.clamp(-1., 1.);

                pellets_to_remove.push(next_coord);
            }

            org.sub_energy(1e-4); // movement takes energy

            transform.translation.x = next_coord.x as f32 * params.cell_width;
            transform.translation.y = next_coord.y as f32 * params.cell_height;

            grid.set(coord.x as usize, coord.y as usize, CellType::Empty);
            grid.set(
                next_coord.x as usize,
                next_coord.y as usize,
                CellType::Impassable,
            );

            *coord = next_coord;

            if org.energy > 0.25 {
                let mut child = org.replicate(5e-2);
                if child.genome.get_distance(&org.genome) > 1e-1 {
                    child.species = species.add_species();
                }
                species.increment_species(child.species);
                children.push((child, *coord));
            }
        }

        while let Some(pellet_coord) = pellets_to_remove.pop() {
            for (e, coord, _, _) in pellets_query.iter() {
                if *coord == pellet_coord {
                    commands.entity(e).despawn_recursive();
                }
            }
        }

        for (child, parent_coord) in children.into_iter() {
            let nearby_coords = grid.clone().search_area(parent_coord, 1, CellType::Empty);

            if nearby_coords.len() == 0 {
                continue;
            }

            let child_coord = nearby_coords[rng.gen_range(0..nearby_coords.len())];

            grid.set(
                child_coord.x as usize,
                child_coord.y as usize,
                CellType::Impassable,
            );

            spawn_organism(
                &mut commands,
                &mut meshes,
                &mut materials,
                &child,
                &child_coord,
                species.get_color(child.species),
                &params,
            );
        }
    }
}
