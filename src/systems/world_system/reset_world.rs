use crate::components::Organism;
use crate::resources::*;
use crate::systems::*;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

pub fn reset_world(
    params: Res<Parameters>,
    mut sim_state: ResMut<SimState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    orgs_query: Query<(Entity, &Organism)>,
    pellets_query: Query<(Entity, With<Pellet>, Without<Organism>)>,
) {
    if sim_state.reset {
        for (e, _) in orgs_query.iter() {
            commands.entity(e).despawn_recursive();
        }
        commands.remove_resource::<Grid>();

        let (orgs, coords, mut grid) = init_world(*params);
        for (org, coord) in orgs.iter().zip(coords.iter()) {
            commands.spawn((
                org.to_owned(),
                coord.to_owned(),
                MaterialMesh2dBundle {
                    mesh: meshes
                        .add(
                            shape::Quad::new(Vec2 {
                                x: params.cell_width,
                                y: params.cell_width,
                            })
                            .into(),
                        )
                        .into(),
                    material: materials.add(ColorMaterial::from(Color::WHITE)),
                    transform: Transform::from_translation(Vec3::new(
                        (coord.x as f32) * params.cell_width,
                        (coord.y as f32) * params.cell_height,
                        0.,
                    )),
                    ..default()
                },
            ));
        }

        for (e, _, _) in pellets_query.iter() {
            commands.entity(e).despawn_recursive();
        }

        let pellet_coords = generate_pellets(params.n_initial_entities, &grid);
        for coord in pellet_coords {
            grid.data[[coord.x as usize, coord.y as usize]] = CellType::Consumable;

            commands.spawn((
                Pellet,
                coord.to_owned(),
                MaterialMesh2dBundle {
                    mesh: meshes
                        .add(
                            shape::Quad::new(Vec2 {
                                x: params.cell_width,
                                y: params.cell_width,
                            })
                            .into(),
                        )
                        .into(),
                    material: materials.add(ColorMaterial::from(Color::GREEN)),
                    transform: Transform::from_translation(Vec3::new(
                        (coord.x as f32) * params.cell_width,
                        (coord.y as f32) * params.cell_height,
                        0.,
                    )),
                    ..default()
                },
            ));
        }

        commands.insert_resource(grid);

        sim_state.epoch = 0;
        sim_state.reset = false;
    }
}
