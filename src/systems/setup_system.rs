use super::*;
use crate::components::NsShape;
use crate::resources::*;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub fn setup_sim(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
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

    let mut window = window_query.get_single_mut().unwrap();
    window.resizable = false;

    let grid_size: usize = 300;
    let cell_width = window.width() / grid_size as f32;
    let cell_height = window.height() / grid_size as f32;

    let params = Parameters {
        grid_size,
        n_initial_entities: 100,
        n_max_entities: 500,
        genome_len: 30,
        ns_shape: NsShape::new(N_SENSORS, 15, N_ACTIONS),
        average_lifespan: 10,
        cell_height,
        cell_width,
    };

    commands.insert_resource(params.clone());

    let (orgs, coords, mut grid) = init_world(params);
    for (org, coord) in orgs.iter().zip(coords.iter()) {
        spawn_organism(
            &mut commands,
            &mut meshes,
            &mut materials,
            org,
            coord,
            &params,
        );
    }

    let pellet_coords = generate_pellets(orgs.iter().map(|org| org.energy).sum::<f32>(), &grid);
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

        commands.insert_resource(grid.clone());
    }
}
