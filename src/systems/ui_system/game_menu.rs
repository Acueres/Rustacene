use crate::components::ui::*;
use bevy::prelude::*;

pub fn build_game_menu(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
    //TODO: add more ui elements
    let control_panel = build_control_panel(commands, &asset_server);
    control_panel
}

fn build_control_panel(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
    let info_label = commands
        .spawn((
            TextBundle::from_section(
                "Sim Info",
                TextStyle {
                    font: asset_server.load("fonts/OpenSans-Regular.ttf"),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            ),
            Label,
        ))
        .id();

    let epoch = commands
        .spawn((
            TextBundle::from_sections([
                TextSection::new(
                    "Epoch: ",
                    TextStyle {
                        font: asset_server.load("fonts/OpenSans-Regular.ttf"),
                        font_size: 16.0,
                        color: Color::WHITE,
                    },
                ),
                TextSection::new(
                    "",
                    TextStyle {
                        font: asset_server.load("fonts/OpenSans-Regular.ttf"),
                        font_size: 16.0,
                        color: Color::WHITE,
                    },
                ),
            ]),
            EpochText,
        ))
        .id();

    let population = commands
        .spawn((
            TextBundle::from_sections([
                TextSection::new(
                    "Population: ",
                    TextStyle {
                        font: asset_server.load("fonts/OpenSans-Regular.ttf"),
                        font_size: 16.0,
                        color: Color::WHITE,
                    },
                ),
                TextSection::new(
                    "",
                    TextStyle {
                        font: asset_server.load("fonts/OpenSans-Regular.ttf"),
                        font_size: 16.0,
                        color: Color::WHITE,
                    },
                ),
            ]),
            PopulationText,
        ))
        .id();

    let energy = commands
        .spawn((
            TextBundle::from_sections([
                TextSection::new(
                    "Energy: ",
                    TextStyle {
                        font: asset_server.load("fonts/OpenSans-Regular.ttf"),
                        font_size: 16.0,
                        color: Color::WHITE,
                    },
                ),
                TextSection::new(
                    "",
                    TextStyle {
                        font: asset_server.load("fonts/OpenSans-Regular.ttf"),
                        font_size: 16.0,
                        color: Color::WHITE,
                    },
                ),
            ]),
            EnergyText,
        ))
        .id();

    let info = commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Percent(20.0)),
                justify_content: JustifyContent::SpaceEvenly,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .push_children(&[info_label, epoch, population, energy])
        .id();

    let control_panel = commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(10.0)),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            background_color: Color::DARK_GRAY.into(),
            ..default()
        })
        .push_children(&[info])
        .id();

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::FlexEnd,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .push_children(&[control_panel])
        .id()
}
