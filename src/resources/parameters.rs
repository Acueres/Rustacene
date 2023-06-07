use bevy::prelude::Resource;

#[derive(Resource, Clone, Copy)]
pub struct Parameters {
    pub grid_size: usize,
    pub n_initial_entities: usize,
    pub n_initial_connections: usize,
    pub n_initial_neurons: usize,
    pub mutate_gene_proba: f64,
    pub insert_gene_proba: f64,
    pub delete_gene_proba: f64,
    pub lifespan: usize,
    pub cell_width: f32,
    pub cell_height: f32,
}
