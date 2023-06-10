use crate::components::*;
use crate::resources::Parameters;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use rand::Rng;

pub fn spawn_organism(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    org: &Organism,
    coord: &Coord<isize>,
    color: Color,
    params: &Parameters,
) {
    let dir: Dir = rand::thread_rng().gen();

    let (conn_genes, neuron_genes): (Vec<Gene>, Vec<Gene>) =
        org.genome.iter().partition(|g| g.is_connection());

    let mut neurons_indexed: Vec<_> = neuron_genes
        .into_iter()
        .map(|g| Neuron::from_gene(g))
        .collect();
    neurons_indexed.sort_by(|a, b| a.0.cmp(&b.0));

    let ns_shape = NsShape::new(
        NeuralSystem::N_SENSORS,
        neurons_indexed.len(),
        Action::N_ACTIONS,
    );

    let ns = NeuralSystem::new(
        &neurons_indexed
            .into_iter()
            .map(|(_, memory, neuron)| (memory, neuron))
            .collect::<Vec<(bool, Neuron)>>(),
        &conn_genes
            .iter()
            .map(|gene| Connection::from_gene(*gene, &ns_shape))
            .collect::<Vec<Connection>>(),
        ns_shape,
    );

    commands.spawn((
        org.to_owned(),
        ns,
        coord.to_owned(),
        dir,
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
            material: materials.add(ColorMaterial::from(color)),
            transform: Transform::from_translation(Vec3::new(
                (coord.x as f32) * params.cell_width,
                (coord.y as f32) * params.cell_height,
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
