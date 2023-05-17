use crate::components::Organism;
use crate::resources::*;
use crate::systems::*;
use bevy::prelude::*;

pub fn reset_world(
    params: Res<Parameters>,
    mut sim_state: ResMut<SimState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut species: ResMut<Species>,
    orgs_query: Query<(Entity, &Organism)>,
    pellets_query: Query<(Entity, With<Pellet>, Without<Organism>)>,
) {
    if sim_state.reset {
        for (e, _) in orgs_query.iter() {
            commands.entity(e).despawn_recursive();
        }
        commands.remove_resource::<Grid>();

        let (orgs, new_species, coords, mut grid) = init_world(*params);
        *species = new_species;
        for (org, coord) in orgs.iter().zip(coords.iter()) {
            spawn_organism(
                &mut commands,
                &mut meshes,
                &mut materials,
                org,
                coord,
                species.get_color(org.species),
                &params,
            );
        }

        for (e, _, _) in pellets_query.iter() {
            commands.entity(e).despawn_recursive();
        }

        let pellet_coords = generate_pellets(orgs.iter().map(|org| org.energy).sum::<f32>(), &grid);
        for coord in pellet_coords {
            grid.set(coord.x as usize, coord.y as usize, CellType::Consumable);
            spawn_pellet(
                &mut commands,
                &mut meshes,
                &mut materials,
                &coord,
                params.cell_width,
                params.cell_height,
            );
        }

        commands.insert_resource(grid);

        sim_state.epoch = 0;
        sim_state.reset = false;
    }
}
