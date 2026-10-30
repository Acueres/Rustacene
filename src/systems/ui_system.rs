mod energy_info_system;
mod epoch_info_system;
mod game_menu;
mod population_info_system;
mod species_info_system;

pub use energy_info_system::*;
pub use epoch_info_system::*;
pub use game_menu::*;
pub use population_info_system::*;
pub use species_info_system::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{CellType, ui::*};
    use crate::resources::{Grid, SimState, Species};
    use bevy::prelude::*;
    use std::collections::HashSet;

    #[derive(Resource, Default)]
    struct DisplayChanges {
        texts: Vec<String>,
        colors: usize,
    }

    fn record_display_changes(
        texts: Query<&Text, Changed<Text>>,
        colors: Query<(), Changed<TextColor>>,
        mut changes: ResMut<DisplayChanges>,
    ) {
        changes.texts = texts.iter().map(|text| text.0.clone()).collect();
        changes.colors = colors.iter().count();
    }

    #[test]
    fn unchanged_stats_do_not_invalidate_text() {
        let mut app = App::new();
        app.init_resource::<DisplayChanges>()
            .insert_resource(Grid::new((10, 10)))
            .insert_resource(SimState {
                paused: true,
                reset: false,
                epoch: 0,
            })
            .insert_resource(Species::new(HashSet::from([0])))
            .add_systems(
                Update,
                (
                    (
                        energy_info_system,
                        epoch_info_system,
                        population_info_system,
                        species_info_system,
                    ),
                    record_display_changes,
                )
                    .chain(),
            );
        app.world_mut().spawn((Text::default(), EnergyText));
        app.world_mut().spawn((Text::default(), EpochText));
        app.world_mut().spawn((Text::default(), PopulationText));
        app.world_mut().spawn((Text::default(), TotalSpeciesText));
        app.world_mut()
            .spawn((Text::default(), TextColor::default(), SpeciesText));

        app.update();
        assert_eq!(app.world().resource::<DisplayChanges>().texts.len(), 5);
        app.update();
        let changes = app.world().resource::<DisplayChanges>();
        assert!(changes.texts.is_empty());
        assert_eq!(changes.colors, 0);

        // A new pellet must still refresh the energy display on the next frame.
        app.world_mut()
            .resource_mut::<Grid>()
            .set(0, 0, CellType::Consumable);
        app.update();
        assert_eq!(app.world().resource::<DisplayChanges>().texts, ["0.2"]);
    }
}
