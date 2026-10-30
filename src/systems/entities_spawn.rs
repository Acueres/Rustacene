use crate::components::*;
use crate::resources::Parameters;
use bevy::color::palettes::css::GREEN;
use bevy::prelude::*;

pub fn spawn_organism(
    commands: &mut Commands,
    org: &Organism,
    coord: &Coord<isize>,
    color: Color,
    params: &Parameters,
) {
    let dir: Dir = rand::random();

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
        // Bevy shares sprite geometry and caches materials for identical sprites.
        Sprite {
            alpha_mode: bevy::sprite::SpriteAlphaMode::Opaque,
            ..Sprite::from_color(color, Vec2::splat(params.cell_width))
        },
        Transform::from_translation(Vec3::new(
            (coord.x as f32) * params.cell_width,
            (coord.y as f32) * params.cell_height,
            0.,
        )),
    ));
}

pub fn spawn_pellet(
    commands: &mut Commands,
    coord: &Coord<isize>,
    cell_width: f32,
    cell_height: f32,
) {
    commands.spawn((
        Pellet,
        coord.to_owned(),
        Sprite {
            alpha_mode: bevy::sprite::SpriteAlphaMode::Opaque,
            ..Sprite::from_color(Color::Srgba(GREEN), Vec2::splat(cell_width))
        },
        Transform::from_translation(Vec3::new(
            (coord.x as f32) * cell_width,
            (coord.y as f32) * cell_height,
            0.,
        )),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::AssetApp;
    use bevy::sprite_render::{SpriteMeshMaterial, SpriteMeshPlugin};

    #[test]
    fn pellets_share_render_assets_across_spawns() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<bevy::shader::Shader>()
            .add_plugins((
                bevy::mesh::MeshPlugin,
                bevy::image::TextureAtlasPlugin,
                SpriteMeshPlugin,
            ));

        for x in 0..2 {
            spawn_pellet(&mut app.world_mut().commands(), &Coord::new(x, 0), 2., 3.);
            app.world_mut().flush();
            app.update();
        }

        let world = app.world_mut();
        let mut query = world.query_filtered::<
            (&Mesh2d, &MeshMaterial2d<SpriteMeshMaterial>, &Transform),
            With<Pellet>,
        >();
        let cells: Vec<_> = query.iter(world).collect();
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].0.0.id(), cells[1].0.0.id());
        assert_eq!(cells[0].1.0.id(), cells[1].1.0.id());
        assert_eq!(
            (cells[0].2.translation - cells[1].2.translation).abs(),
            Vec3::new(2., 0., 0.),
        );
    }
}
