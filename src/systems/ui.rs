use crate::organism::Organism;
use crate::systems::*;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

pub fn ui(
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
