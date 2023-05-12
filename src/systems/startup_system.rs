use super::*;
use crate::components::NsShape;
use crate::resources::*;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub fn startup_system(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut window = window_query.get_single_mut().unwrap();
    window.resizable = false;

    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::MIDNIGHT_BLUE),
        },
        transform: Transform::from_translation(Vec3::new(
            window.width() / 2.,
            window.height() / 2.,
            0.,
        )),
        ..Default::default()
    });

    let grid_size: usize = 500;
    let cell_width = window.width() / grid_size as f32;
    let cell_height = window.height() / grid_size as f32;

    let params = Parameters {
        grid_size,
        n_initial_entities: 100,
        n_max_entities: 500,
        genome_len: 20,
        ns_shape: NsShape::new(N_SENSORS, 5, N_ACTIONS),
        average_lifespan: 7,
        cell_height,
        cell_width,
    };

    commands.insert_resource(params.clone());

    let (orgs, initial_species, coords, mut grid) = init_world(params);
    let species = Species::new(initial_species);
    for (org, coord) in orgs.iter().zip(coords.iter()) {
        spawn_organism(
            &mut commands,
            &mut meshes,
            &mut materials,
            org,
            coord,
            species.get_color(org.species),
            &params,
        );
    }

    commands.insert_resource(species);

    let pellet_coords = generate_pellets(orgs.iter().map(|org| org.energy).sum::<f32>(), &grid);
    for coord in pellet_coords.iter() {
        grid.set(coord.x as usize, coord.y as usize, CellType::Consumable);
        spawn_pellet(
            &mut commands,
            &mut meshes,
            &mut materials,
            coord,
            params.cell_width,
            params.cell_height,
        );
    }

    commands.insert_resource(grid.clone());

    let control_menu = build_control_menu(&mut commands, &asset_server);
    commands.entity(control_menu).insert(ControlMenu);
}

fn build_control_menu(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
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
