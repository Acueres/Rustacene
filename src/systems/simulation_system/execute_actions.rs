use super::*;
use crate::components::{Action, Coord, Organism};
use crate::resources::*;
use crate::systems::*;
use bevy::prelude::*;

pub fn execute_actions(
    mut commands: Commands,
    time: Res<Time>,
    sim_state: Res<SimState>,
    params: Res<Parameters>,
    mut sim_time: ResMut<SimTime>,
    mut grid: ResMut<Grid>,
    mut orgs_query: Query<(
        &mut Organism,
        &mut NeuralSystem,
        &mut Coord<isize>,
        &mut Dir,
        &mut Transform,
    )>,
    pellets_query: Query<(Entity, &Coord<isize>, With<Pellet>, Without<Organism>)>,
) {
    if !sim_state.paused && !sim_state.reset && sim_time.timer.tick(time.delta()).just_finished() {
        let mut pellets_to_remove = Vec::<Coord<isize>>::new();

        for (mut org, mut ns, mut coord, mut curr_dir, mut transform) in orgs_query.iter_mut() {
            if org.energy < 0. {
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
        }

        while let Some(pellet_coord) = pellets_to_remove.pop() {
            for (e, coord, _, _) in &pellets_query {
                if *coord == pellet_coord {
                    commands.entity(e).despawn_recursive();
                }
            }
        }
    }
}
