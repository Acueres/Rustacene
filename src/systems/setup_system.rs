use super::*;
use crate::models::NsShape;
use crate::resources::*;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

pub fn setup_sim(
    mut windows: ResMut<Windows>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::MIDNIGHT_BLUE),
        },
        ..Default::default()
    });

    for window in windows.iter_mut() {
        window.set_resizable(false);

        let grid_size: usize = 300;
        let cell_width = window.width() / grid_size as f32;
        let cell_height = window.height() / grid_size as f32;

        let params = Parameters {
            grid_size,
            n_initial_entities: 100,
            n_max_entities: 500,
            genome_len: 15,
            ns_shape: NsShape::new(4, 5, 12), //in = x, y, pellet_x, pellet_y, internal = 1, out = 9 dirs
            average_lifespan: 7,
            cell_height,
            cell_width,
        };

        commands.insert_resource(params.clone());

        let (orgs, coords, mut grid) = init_world(params);
        for (org, coord) in orgs.iter().zip(coords.iter()) {
            commands.spawn((
                org.to_owned(),
                coord.to_owned(),
                MaterialMesh2dBundle {
                    mesh: meshes
                        .add(
                            shape::Quad::new(Vec2 {
                                x: cell_width,
                                y: cell_width,
                            })
                            .into(),
                        )
                        .into(),
                    material: materials.add(ColorMaterial::from(Color::WHITE)),
                    transform: Transform::from_translation(Vec3::new(
                        (coord.x as f32) * cell_width,
                        (coord.y as f32) * cell_height,
                        0.,
                    )),
                    ..default()
                },
            ));
        }

        let pellet_coords = generate_pellets(params.n_initial_entities, grid.to_owned());
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
    }
}
