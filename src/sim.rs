use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
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
}

#[derive(Component, Clone)]
pub struct Particle;

#[derive(Component, Clone, PartialEq)]
struct Coord<T> {
    pub x: T,
    pub y: T,
}

#[derive(Component)]
struct Grid {
    pub data: Vec<Vec<bool>>,
}

#[derive(Component, Clone)]
struct Parameters {
    pub grid_size: usize,
    pub n_particles: usize,
}

struct SimTimer(Timer);

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimTimer(Timer::from_seconds(0.02, true)))
            .add_startup_system(setup)
            .add_system(sim_step)
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
            grid_size: 100,
            n_particles: 20,
        };
        commands.spawn().insert(params.clone());

        let particle_width = window.width() / params.grid_size as f32;
        let particle_height = window.height() / params.grid_size as f32;
        let particle_size = 1.5 * particle_height;

        let mut grid = vec![vec![false; params.grid_size]; params.grid_size];

        let mut rng = rand::thread_rng();

        let mut n = 0;
        while n < params.n_particles {
            let x = rng.gen_range(0..params.grid_size);
            let y = rng.gen_range(0..params.grid_size);

            if grid[x][y] {
                continue;
            }

            grid[x][y] = true;

            commands
                .spawn()
                .insert(Particle)
                .insert(Coord::<isize> {
                    x: x as isize,
                    y: y as isize,
                })
                .insert_bundle(MaterialMesh2dBundle {
                    mesh: meshes
                        .add(
                            shape::Quad::new(Vec2 {
                                x: particle_size,
                                y: particle_size,
                            })
                            .into(),
                        )
                        .into(),
                    material: materials.add(ColorMaterial::from(Color::WHITE)),
                    transform: Transform::from_translation(Vec3::new(
                        (x as f32) * particle_width,
                        (y as f32) * particle_height,
                        0.,
                    )),
                    ..default()
                });

            n += 1;
        }

        commands.spawn().insert(Grid { data: grid });
    }
}

fn sim_step(
    time: Res<Time>,
    windows: Res<Windows>,
    mut timer: ResMut<SimTimer>,
    mut particles_query: Query<(&mut Particle, &mut Coord<isize>, &mut Transform)>,
    mut grid_query: Query<&mut Grid>,
    params_query: Query<&Parameters>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for window in windows.iter() {
            let mut rng = rand::thread_rng();

            let mut grid = grid_query.single_mut();
            let params = params_query.single();

            let particle_width = window.width() / params.grid_size as f32;
            let particle_height = window.height() / params.grid_size as f32;

            for (mut p, mut coord, mut transform) in particles_query.iter_mut() {
                let next_dir: Dir = rng.gen();
                let dir_coord: Coord<i8> = next_dir.value();
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
                if *coord != next_coord && grid.data[next_coord.x as usize][next_coord.y as usize] {
                    continue;
                }

                transform.translation.x = next_coord.x as f32 * particle_width;
                transform.translation.y = next_coord.y as f32 * particle_height;

                grid.data[coord.x as usize][coord.y as usize] = false;
                grid.data[next_coord.x as usize][next_coord.y as usize] = true;

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

#[inline]
fn position_to_coord(pos: (f32, f32), particle_width: f32, particle_height: f32) -> Coord<usize> {
    Coord {
        x: (pos.0 / particle_width) as usize,
        y: (pos.1 / particle_height) as usize,
    }
}
