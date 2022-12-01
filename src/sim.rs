use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::utils::Duration;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use ndarray::Array2;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

#[derive(Copy, Clone)]
enum Dir {
    NULL,
    N,
    S,
    E,
    W,
    NE,
    NW,
    SE,
    SW,
}

impl Distribution<Dir> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Dir {
        match rng.gen_range(0..9) {
            0 => Dir::NULL,
            1 => Dir::N,
            2 => Dir::S,
            3 => Dir::E,
            4 => Dir::W,
            5 => Dir::NE,
            6 => Dir::NW,
            7 => Dir::SE,
            _ => Dir::SW,
        }
    }
}

impl Dir {
    pub fn value(self) -> Coord<i8> {
        match self {
            Self::NULL => Coord { x: 0, y: 0 },
            Self::N => Coord { x: 0, y: 1 },
            Self::S => Coord { x: 0, y: -1 },
            Self::E => Coord { x: 1, y: 0 },
            Self::W => Coord { x: -1, y: 0 },
            Self::NE => Coord { x: 1, y: 1 },
            Self::NW => Coord { x: -1, y: 1 },
            Self::SE => Coord { x: 1, y: -1 },
            Self::SW => Coord { x: -1, y: -1 },
        }
    }

    pub fn get(index: usize) -> Self {
        match index {
            0 => Self::NULL,
            1 => Self::N,
            2 => Self::S,
            3 => Self::E,
            4 => Self::W,
            5 => Self::NE,
            6 => Self::NW,
            7 => Self::SE,
            8 => Self::SW,
            _ => Self::NULL,
        }
    }
}

#[derive(Component, Clone)]
struct Cell {
    pub genome: Vec<f32>,
}

#[derive(Component, Clone, PartialEq)]
struct Coord<T> {
    pub x: T,
    pub y: T,
}

#[derive(Component, Clone)]
struct Grid {
    pub data: Array2<bool>,
}

#[derive(Component, Clone)]
struct Parameters {
    pub grid_size: usize,
    pub n_entities: usize,
    pub genome_len: usize,
}

struct SimTimer(Timer);

#[derive(Component)]
struct SimState {
    pub paused: bool,
    pub reset: bool,
}

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimTimer(Timer::from_seconds(0.05, true)))
            .insert_resource(SimState {
                paused: false,
                reset: false,
            })
            .add_startup_system(setup)
            .add_system(handle_input)
            .add_system(sim_step)
            .add_system(reset_sim)
            .add_system(center_camera);
    }
}

fn setup(
    mut windows: ResMut<Windows>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::MIDNIGHT_BLUE),
        },
        ..Default::default()
    });

    for window in windows.iter_mut() {
        window.set_resizable(false);

        let params = Parameters {
            grid_size: 300,
            n_entities: 100,
            genome_len: 2 + 9,
        };

        commands.insert_resource(params.clone());

        let cell_width = window.width() / params.grid_size as f32;
        let cell_height = window.height() / params.grid_size as f32;
        let cell_size = 1.5 * cell_height;

        let (cells, coords, grid) =
            generate_entities(params.n_entities, params.grid_size, params.genome_len);
        for (cell, coord) in cells.iter().zip(coords.iter()) {
            commands
                .spawn()
                .insert(cell.to_owned())
                .insert(coord.to_owned())
                .insert_bundle(MaterialMesh2dBundle {
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
                });
        }

        commands.insert_resource(grid);
    }
}

fn sim_step(
    time: Res<Time>,
    windows: Res<Windows>,
    sim_state: Res<SimState>,
    params: Res<Parameters>,
    mut timer: ResMut<SimTimer>,
    mut grid: ResMut<Grid>,
    mut entities_query: Query<(&Cell, &mut Coord<isize>, &mut Transform)>,
) {
    if !sim_state.paused && !sim_state.reset && timer.0.tick(time.delta()).just_finished() {
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
                let dir_coord: Coord<i8> = dir.value();
                let next_coord = Coord {
                    x: coord.x + dir_coord.x as isize,
                    y: coord.y + dir_coord.y as isize,
                };

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
            for e in entities_query.iter() {
                commands.entity(e.0).despawn_recursive();
            }
            commands.remove_resource::<Grid>();

            let cell_width = window.width() / params.grid_size as f32;
            let cell_height = window.height() / params.grid_size as f32;
            let cell_size = 1.5 * cell_height;

            let (cells, coords, grid) =
                generate_entities(params.n_entities, params.grid_size, params.genome_len);
            for (cell, coord) in cells.iter().zip(coords.iter()) {
                commands
                    .spawn()
                    .insert(cell.to_owned())
                    .insert(coord.to_owned())
                    .insert_bundle(MaterialMesh2dBundle {
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
                    });
            }

            commands.insert_resource(grid);

            sim_state.reset = false;
        }
    }
}

fn handle_input(
    keys: Res<Input<KeyCode>>,
    mut timer: ResMut<SimTimer>,
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
        timer.0.set_duration(Duration::from_secs_f32(0.1));
    }
    if keys.just_pressed(KeyCode::Key2) {
        timer.0.set_duration(Duration::from_secs_f32(0.05));
    }
    if keys.just_pressed(KeyCode::Key3) {
        timer.0.set_duration(Duration::from_secs_f32(0.025));
    }
}

fn generate_entities(
    n_entities: usize,
    grid_size: usize,
    genome_len: usize,
) -> (Vec<Cell>, Vec<Coord<isize>>, Grid) {
    let mut cells = Vec::<Cell>::new();
    cells.reserve_exact(n_entities);

    let mut coords = Vec::<Coord<isize>>::new();
    coords.reserve_exact(n_entities);

    let v = vec![false; grid_size * grid_size];
    let mut grid = Grid {
        data: Array2::<bool>::from_shape_vec((grid_size, grid_size), v).unwrap(),
    };

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
        cells.push(Cell { genome });

        let coord = Coord::<isize> {
            x: x as isize,
            y: y as isize,
        };
        coords.push(coord);

        n += 1;
    }
    (cells, coords, grid)
}
