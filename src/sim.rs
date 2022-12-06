use crate::cell::Cell;
use crate::coord::Coord;
use crate::dir::Dir;
use crate::grid::Grid;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::utils::Duration;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_egui::{egui, EguiContext};
use rand::Rng;

#[derive(Resource, Clone)]
struct Parameters {
    pub grid_size: usize,
    pub n_initial_entities: usize,
    pub n_max_entities: usize,
    pub genome_len: usize,
    pub cell_lifespan: usize,
}

#[derive(Resource)]
struct SimTime {
    pub timer: Timer,
}

#[derive(Resource)]
struct EpochTime {
    pub timer: Timer,
}

#[derive(Resource)]
struct SimState {
    pub paused: bool,
    pub reset: bool,
    pub epoch: usize,
}

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimTime {
            timer: Timer::from_seconds(0.05, TimerMode::Repeating),
        })
        .insert_resource(EpochTime {
            timer: Timer::from_seconds(5., TimerMode::Repeating),
        })
        .insert_resource(SimState {
            paused: false,
            reset: false,
            epoch: 0,
        })
        .add_startup_system(setup)
        .add_system(handle_input)
        .add_system(execute_actions)
        .add_system(advance_epoch)
        .add_system(reset_sim)
        .add_system(center_camera)
        .add_system(ui);
    }
}

fn setup(
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

        let params = Parameters {
            grid_size: 300,
            n_initial_entities: 100,
            n_max_entities: 500,
            genome_len: 2 + 9,
            cell_lifespan: 3,
        };

        commands.insert_resource(params.clone());

        let cell_width = window.width() / params.grid_size as f32;
        let cell_height = window.height() / params.grid_size as f32;
        let cell_size = 1.5 * cell_height;

        let (cells, coords, grid) = generate_entities(
            params.n_initial_entities,
            params.grid_size,
            params.genome_len,
        );
        for (cell, coord) in cells.iter().zip(coords.iter()) {
            commands.spawn((
                cell.to_owned(),
                coord.to_owned(),
                MaterialMesh2dBundle {
                    mesh: meshes
                        .add(
                            shape::Quad::new(Vec2 {
                                x: cell_size,
                                y: cell_size,
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

        commands.insert_resource(grid);
    }
}

fn execute_actions(
    time: Res<Time>,
    windows: Res<Windows>,
    sim_state: Res<SimState>,
    params: Res<Parameters>,
    mut sim_time: ResMut<SimTime>,
    mut grid: ResMut<Grid>,
    mut entities_query: Query<(&Cell, &mut Coord<isize>, &mut Transform)>,
) {
    if !sim_state.paused && !sim_state.reset && sim_time.timer.tick(time.delta()).just_finished() {
        for window in windows.iter() {
            let mut rng = rand::thread_rng();

            let cell_width = window.width() / params.grid_size as f32;
            let cell_height = window.height() / params.grid_size as f32;

            for (cell, mut coord, mut transform) in entities_query.iter_mut() {
                let x_coord = coord.x as f32 / params.grid_size as f32;
                let y_coord = coord.y as f32 / params.grid_size as f32;
                let probas: Vec<f32> = vec![
                    (cell.genome[0] * (x_coord + y_coord)).tanh(),
                    (cell.genome[1] * (2. - x_coord - y_coord)).tanh(),
                ];

                let fire_values: Vec<bool> = cell.genome[2..]
                    .iter()
                    .map(|w| (probas.iter().sum::<f32>() * w).tanh())
                    .collect::<Vec<f32>>()
                    .iter()
                    .map(|p| {
                        if p.is_sign_negative() {
                            false
                        } else {
                            rng.gen_bool(p.to_owned() as f64)
                        }
                    })
                    .collect();

                let mut actions = Vec::<usize>::new();
                for (i, fire) in fire_values.iter().enumerate() {
                    if fire.to_owned() {
                        actions.push(i);
                    }
                }

                let action_index = if actions.len() == 0 {
                    0
                } else {
                    actions[rng.gen_range(0..actions.len())]
                };

                let dir: Dir = Dir::get(action_index);
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
                if *coord != next_coord && grid.data[[next_coord.x as usize, next_coord.y as usize]]
                {
                    continue;
                }

                transform.translation.x = next_coord.x as f32 * cell_width;
                transform.translation.y = next_coord.y as f32 * cell_height;

                grid.data[[coord.x as usize, coord.y as usize]] = false;
                grid.data[[next_coord.x as usize, next_coord.y as usize]] = true;

                *coord = next_coord;
            }
        }
    }
}

fn advance_epoch(
    windows: Res<Windows>,
    time: Res<Time>,
    mut sim_state: ResMut<SimState>,
    params: Res<Parameters>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut epoch_time: ResMut<EpochTime>,
    mut grid: ResMut<Grid>,
    mut entities_query: Query<(Entity, &mut Cell, &Coord<isize>)>,
) {
    if !sim_state.paused && !sim_state.reset && epoch_time.timer.tick(time.delta()).just_finished()
    {
        for window in windows.iter() {
            let cell_width = window.width() / params.grid_size as f32;
            let cell_height = window.height() / params.grid_size as f32;
            let cell_size = 1.5 * cell_height;

            let mut rng = rand::thread_rng();
            let mut children = Vec::<(Cell, Coord<isize>)>::new();

            sim_state.epoch += 1;

            let mut n_entities = entities_query.iter().len();

            for (e, mut cell, coord) in entities_query.iter_mut() {
                cell.age += 1;

                if cell.age == params.cell_lifespan {
                    grid.data[[coord.x as usize, coord.y as usize]] = false;
                    commands.entity(e).despawn_recursive();
                    n_entities -= 1;
                    continue;
                }

                if n_entities >= params.n_max_entities {
                    continue;
                }

                let free_coords = grid.clone().get_free_coords(coord.to_owned());
                if free_coords.len() > 0 {
                    let child_coord = free_coords[rng.gen_range(0..free_coords.len())];
                    let child = cell.clone().replicate(0.05);
                    children.push((child, child_coord));
                    grid.data[[child_coord.x as usize, child_coord.y as usize]] = true;
                    n_entities += 1;
                }
            }

            for (cell, coord) in children {
                commands.spawn((
                    cell.to_owned(),
                    coord.to_owned(),
                    MaterialMesh2dBundle {
                        mesh: meshes
                            .add(
                                shape::Quad::new(Vec2 {
                                    x: cell_size,
                                    y: cell_size,
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
        }
    }
}

fn center_camera(mut windows: ResMut<Windows>, mut camera: Query<&mut Transform, With<Camera>>) {
    for window in windows.iter_mut() {
        for mut transform in camera.iter_mut() {
            transform.translation.x = window.width() / 2.;
            transform.translation.y = window.height() / 2.;
        }
    }
}

fn reset_sim(
    windows: Res<Windows>,
    params: Res<Parameters>,
    mut sim_state: ResMut<SimState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    entities_query: Query<(Entity, &Cell)>,
) {
    if sim_state.reset {
        for window in windows.iter() {
            for (e, _) in entities_query.iter() {
                commands.entity(e).despawn_recursive();
            }
            commands.remove_resource::<Grid>();

            let cell_width = window.width() / params.grid_size as f32;
            let cell_height = window.height() / params.grid_size as f32;
            let cell_size = 1.5 * cell_height;

            let (cells, coords, grid) = generate_entities(
                params.n_initial_entities,
                params.grid_size,
                params.genome_len,
            );
            for (cell, coord) in cells.iter().zip(coords.iter()) {
                commands.spawn((
                    cell.to_owned(),
                    coord.to_owned(),
                    MaterialMesh2dBundle {
                        mesh: meshes
                            .add(
                                shape::Quad::new(Vec2 {
                                    x: cell_size,
                                    y: cell_size,
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

            commands.insert_resource(grid);

            sim_state.epoch = 0;
            sim_state.reset = false;
        }
    }
}

fn handle_input(
    keys: Res<Input<KeyCode>>,
    mut sim_time: ResMut<SimTime>,
    mut sim_state: ResMut<SimState>,
) {
    if keys.just_pressed(KeyCode::Space) {
        sim_state.paused ^= true;
    }
    if keys.just_pressed(KeyCode::R) {
        sim_state.reset = true;
    }

    //sim speed control
    if keys.just_pressed(KeyCode::Key1) {
        sim_time.timer.set_duration(Duration::from_secs_f32(0.1));
    }
    if keys.just_pressed(KeyCode::Key2) {
        sim_time.timer.set_duration(Duration::from_secs_f32(0.05));
    }
    if keys.just_pressed(KeyCode::Key3) {
        sim_time.timer.set_duration(Duration::from_secs_f32(0.025));
    }
}

fn generate_entities(
    n_entities: usize,
    grid_size: usize,
    genome_len: usize,
) -> (Vec<Cell>, Vec<Coord<isize>>, Grid) {
    let mut cells = Vec::<Cell>::new();
    cells.reserve_exact(n_entities * 3);

    let mut coords = Vec::<Coord<isize>>::new();
    coords.reserve_exact(n_entities * 3);

    let mut grid = Grid::init((grid_size, grid_size));

    let mut rng = rand::thread_rng();

    let mut n = 0;
    while n < n_entities {
        let x = rng.gen_range(0..grid_size);
        let y = rng.gen_range(0..grid_size);

        if grid.data[[x, y]] {
            continue;
        }

        grid.data[[x, y]] = true;

        let genome = vec![0.; genome_len]
            .iter()
            .map(|_| rng.gen_range(-1.0..1.))
            .collect();
        cells.push(Cell { genome, age: 0 });

        let coord = Coord::<isize> {
            x: x as isize,
            y: y as isize,
        };
        coords.push(coord);

        n += 1;
    }
    (cells, coords, grid)
}

fn ui(mut egui_context: ResMut<EguiContext>, sim_state: Res<SimState>, cells_query: Query<&Cell>) {
    egui::Window::new("Info")
        .fixed_pos(egui::Pos2::new(0., 0.))
        .show(egui_context.ctx_mut(), |ui| {
            ui.label(format!("Epoch {}", sim_state.epoch));
            ui.label(format!("N {}", cells_query.iter().len()));
        });
}
