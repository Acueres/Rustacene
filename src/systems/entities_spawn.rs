use crate::components::*;
use crate::resources::Parameters;
use bevy::prelude::*;
use bevy_color::palettes::css::GREEN;
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

    let mut neurons: Vec<_> = neuron_genes
        .into_iter()
        .map(|g| Neuron::from_gene(g))
        .collect();
    neurons.sort_by(|a, b| a.0.cmp(&b.0));

    let (sensor_neurons, hidden_neurons) = neurons.split_at(SensorySystem::N_SENSORS);

    let ss = SensorySystem::new(
        sensor_neurons
            .into_iter()
            .map(|(_, _, neuron)| (neuron.w - 0.5) * 2.)
            .collect(),
    );

    let ns_shape = NsShape::new(
        SensorySystem::N_SENSORS,
        hidden_neurons.len(),
        Action::N_ACTIONS,
    );
    let ns = NeuralSystem::new(
        &hidden_neurons
            .into_iter()
            .map(|(_, memory, neuron)| (*memory, *neuron))
            .collect(),
        &conn_genes
            .iter()
            .map(|gene| Connection::from_gene(*gene, &ns_shape))
            .collect(),
        ns_shape,
    );

    commands.spawn((
        org.to_owned(),
        ss,
        ns,
        coord.to_owned(),
        dir,
        Mesh2d(meshes.add(Rectangle::new(params.cell_width, params.cell_width))),
        MeshMaterial2d(materials.add(ColorMaterial::from(color))),
        Transform::from_translation(Vec3::new(
            (coord.x as f32) * params.cell_width,
            (coord.y as f32) * params.cell_height,
            0.,
        )),
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
        Mesh2d(meshes.add(Rectangle::new(cell_width, cell_width))),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::Srgba(GREEN)))),
        Transform::from_translation(Vec3::new(
            (coord.x as f32) * cell_width,
            (coord.y as f32) * cell_height,
            0.,
        )),
    ));
}
