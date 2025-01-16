use crate::components::ui::*;
use bevy::prelude::*;
use bevy_color::palettes::css::WHITE;

pub fn build_game_menu(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
    //TODO: add more ui elements
    let control_panel = build_control_panel(commands, &asset_server);
    control_panel
}

fn build_control_panel(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
    let info = build_info_panel(commands, asset_server);
    let species = build_species_panel(commands, asset_server);

    let control_panel = commands
        .spawn(Node {
            width: Val::Percent(20.0),
            height: Val::Percent(10.0),
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .add_children(&[info, species])
        .id();

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::FlexEnd,
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .add_children(&[control_panel])
        .id()
}

fn build_info_panel(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
    let info_label = commands
        .spawn((
            Text::new("Sim Info"),
            TextFont {
                font: asset_server.load("fonts/OpenSans-Regular.ttf"),
                font_size: 20.0,
                ..default()
            },
            Label,
        ))
        .id();

    let epoch = commands
        .spawn((
            Text::new(""),
            TextFont {
                font: asset_server.load("fonts/OpenSans-Regular.ttf"),
                font_size: 16.0,
                ..default()
            },
            EpochText,
        ))
        .id();

    let population = commands
        .spawn((
            Text::new(""),
            TextFont {
                font: asset_server.load("fonts/OpenSans-Regular.ttf"),
                font_size: 16.0,
                ..default()
            },
            PopulationText,
        ))
        .id();

    let energy = commands
        .spawn((
            Text::new(""),
            TextFont {
                font: asset_server.load("fonts/OpenSans-Regular.ttf"),
                font_size: 16.0,
                ..default()
            },
            EnergyText,
        ))
        .id();

    commands
        .spawn(Node {
            width: Val::Percent(50.0),
            justify_content: JustifyContent::SpaceEvenly,
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .add_children(&[info_label, epoch, population, energy])
        .id()
}

fn build_species_panel(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
    let info_label = commands
        .spawn((
            Text::new(""),
            TextFont {
                font: asset_server.load("fonts/OpenSans-Regular.ttf"),
                font_size: 20.0,
                ..default()
            },
            TotalSpeciesText,
        ))
        .id();

    let children = &[
        info_label,
        spawn_species_label(commands, asset_server),
        spawn_species_label(commands, asset_server),
        spawn_species_label(commands, asset_server),
        spawn_species_label(commands, asset_server),
        spawn_species_label(commands, asset_server),
    ];

    commands
        .spawn(Node {
            width: Val::Percent(50.0),
            justify_content: JustifyContent::SpaceEvenly,
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .add_children(children)
        .id()
}

fn spawn_species_label(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
    commands
        .spawn((
            Text::new(""),
            TextFont {
                font: asset_server.load("fonts/OpenSans-Regular.ttf"),
                font_size: 14.0,
                ..default()
            },
            TextColor(WHITE.into()),
            SpeciesText,
        ))
        .id()
}
