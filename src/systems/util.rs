use crate::components::{Coord, Organism, Pellet};
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

pub fn spawn_organism(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    org: &Organism,
    coord: &Coord<isize>,
    cell_width: f32,
    cell_height: f32,
) {
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

pub fn spawn_pellet(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    coord: &Coord<isize>,
    cell_width: f32,
    cell_height: f32,
) {
    commands.spawn((
        Pellet,
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
            material: materials.add(ColorMaterial::from(Color::GREEN)),
            transform: Transform::from_translation(Vec3::new(
                (coord.x as f32) * cell_width,
                (coord.y as f32) * cell_height,
                0.,
            )),
            ..default()
        },
    ));
}
