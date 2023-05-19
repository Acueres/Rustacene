use super::*;
use crate::components::ui::*;

pub fn ui_startup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let game_menu = build_game_menu(&mut commands, &asset_server);
    commands.entity(game_menu).insert(GameMenu);
}
