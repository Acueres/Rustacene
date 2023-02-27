use crate::action::Action;
use crate::coord::Coord;
use crate::dir::Dir;
use crate::grid::{CellType, Grid};
use crate::ns::NsShape;
use crate::organism::Organism;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::utils::Duration;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_egui::{egui, EguiContext};
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Resource, Clone, Copy)]
struct Parameters {
    pub grid_size: usize,
    pub n_initial_entities: usize,
    pub n_max_entities: usize,
    pub genome_len: usize,
    pub ns_shape: NsShape,
    pub average_lifespan: usize,
    pub cell_width: f32,
    pub cell_height: f32,
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

#[derive(Component, Clone, Copy)]
struct Pellet;

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

fn execute_actions(
    mut commands: Commands,
    time: Res<Time>,
    sim_state: Res<SimState>,
    params: Res<Parameters>,
    mut sim_time: ResMut<SimTime>,
    mut grid: ResMut<Grid>,
    mut orgs_query: Query<(&mut Organism, &mut Coord<isize>, &mut Transform)>,
    pellets_query: Query<(Entity, &Coord<isize>, With<Pellet>, Without<Organism>)>,
) {
    if !sim_state.paused && !sim_state.reset && sim_time.timer.tick(time.delta()).just_finished() {
        let mut pellets_to_remove = Vec::<Coord<isize>>::new();

        for (mut org, mut coord, mut transform) in orgs_query.iter_mut() {
            org.energy -= 1e-6;
            if org.energy < 0. {
                continue;
            }

            let inputs = process_sensors(
                &coord.to_owned(),
                &grid.to_owned(),
                &params.to_owned(),
                org.direction,
            );

            let action = org.get_action(inputs);
            if action == Action::Halt {
                continue;
            }

            let dir = action.get_dir(org.direction);
            org.direction = dir;
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
            if grid.data[[next_coord.x as usize, next_coord.y as usize]] == CellType::Impassable {
                continue;
            } else if grid.data[[next_coord.x as usize, next_coord.y as usize]]
                == CellType::Consumable
            {
                org.energy += 0.2;
                org.energy = org.energy.clamp(f32::NEG_INFINITY, 1.);

                pellets_to_remove.push(next_coord);
            }

            org.energy -= 1e-4;

            transform.translation.x = next_coord.x as f32 * params.cell_width;
            transform.translation.y = next_coord.y as f32 * params.cell_height;

            grid.data[[coord.x as usize, coord.y as usize]] = CellType::Empty;
            grid.data[[next_coord.x as usize, next_coord.y as usize]] = CellType::Impassable;

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

fn advance_epoch(
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
                grid.data[[coord.x as usize, coord.y as usize]] = CellType::Empty;
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
                grid.data[[child_coord.x as usize, child_coord.y as usize]] = CellType::Impassable;
                n_entities += 1;
            }
        }

        for (org, coord) in children {
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

        if n_entities == 0 {
            return;
        }

        let pellet_coords = generate_pellets(n_entities, grid.to_owned());
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

        sim_state.epoch = 0;
        sim_state.reset = false;
    }
}

#[inline]
fn process_sensors(coord: &Coord<isize>, grid: &Grid, params: &Parameters, dir: Dir) -> Vec<f32> {
    let x_coord = coord.x as f32 / params.grid_size as f32;
    let y_coord = coord.y as f32 / params.grid_size as f32;

    let pellet_coord = grid.search_along_dir(
        coord.x as usize,
        coord.y as usize,
        3,
        dir,
        CellType::Consumable,
    );
    let x_pellet = 1. - (x_coord - (pellet_coord.x as f32 / params.grid_size as f32));
    let y_pellet = 1. - (y_coord - (pellet_coord.y as f32 / params.grid_size as f32));

    vec![x_coord, y_coord, x_pellet, y_pellet]
}

fn handle_input(
    keys: Res<Input<KeyCode>>,
    mut sim_time: ResMut<SimTime>,
    mut epoch_time: ResMut<EpochTime>,
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
        epoch_time.timer.set_duration(Duration::from_secs_f32(10.));
    }
    if keys.just_pressed(KeyCode::Key2) {
        sim_time.timer.set_duration(Duration::from_secs_f32(0.05));
        epoch_time.timer.set_duration(Duration::from_secs_f32(5.));
    }
    if keys.just_pressed(KeyCode::Key3) {
        sim_time.timer.set_duration(Duration::from_secs_f32(0.025));
        epoch_time.timer.set_duration(Duration::from_secs_f32(2.5));
    }
}

fn init_world(params: Parameters) -> (Vec<Organism>, Vec<Coord<isize>>, Grid) {
    let mut orgs = Vec::<Organism>::new();
    orgs.reserve_exact(params.n_initial_entities * 3);

    let mut coords = Vec::<Coord<isize>>::new();
    coords.reserve_exact(params.n_initial_entities * 3);

    let mut grid = Grid::init((params.grid_size, params.grid_size));

    let mut rng = rand::thread_rng();

    let mut n = 0;
    while n < params.n_initial_entities {
        let x = rng.gen_range(0..params.grid_size);
        let y = rng.gen_range(0..params.grid_size);

        if grid.data[[x, y]] == CellType::Impassable {
            continue;
        }

        grid.data[[x, y]] = CellType::Impassable;

        orgs.push(Organism::new(0.5, params.genome_len, params.ns_shape));

        let coord = Coord::<isize> {
            x: x as isize,
            y: y as isize,
        };
        coords.push(coord);

        n += 1;
    }
    (orgs, coords, grid)
}

fn generate_pellets(n_entities: usize, grid: Grid) -> Vec<Coord<isize>> {
    let rng = &mut rand::thread_rng();
    let n_pellets = (100 * (250 / n_entities)).clamp(0, n_entities);

    grid.data
        .indexed_iter()
        .filter(|x| x.1.to_owned() == CellType::Empty)
        .collect::<Vec<((usize, usize), &CellType)>>()
        .iter()
        .map(|v| Coord::<isize> {
            x: v.0 .0 as isize,
            y: v.0 .1 as isize,
        })
        .collect::<Vec<Coord<isize>>>()
        .choose_multiple(rng, n_pellets)
        .cloned()
        .collect::<Vec<Coord<isize>>>()
}

fn ui(
    mut egui_context: ResMut<EguiContext>,
    sim_state: Res<SimState>,
    orgs_query: Query<&Organism>,
) {
    egui::Window::new("Info")
        .fixed_pos(egui::Pos2::new(0., 0.))
        .show(egui_context.ctx_mut(), |ui| {
            ui.label(format!("Epoch {}", sim_state.epoch));
            ui.label(format!("N {}", orgs_query.iter().len()));
        });
}
