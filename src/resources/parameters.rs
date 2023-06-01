use crate::components::NsShape;
use bevy::prelude::Resource;

#[derive(Resource, Clone, Copy)]
pub struct Parameters {
    pub grid_size: usize,
    pub n_initial_entities: usize,
    pub initial_genome_len: usize,
    pub mutate_gene_proba: f64,
    pub insert_gene_proba: f64,
    pub delete_gene_proba: f64,
    pub ns_shape: NsShape,
    pub average_lifespan: usize,
    pub cell_width: f32,
    pub cell_height: f32,
}
