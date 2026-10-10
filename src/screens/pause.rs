use raylib::prelude::*;

use crate::game::Game;
use crate::game_mode::GameMode;
use crate::input;
use crate::ui::HudRects;

pub fn handle_escape(game: &mut Game, rl: &RaylibHandle) {
    if !input::pause_pressed(rl) {
        return;
    }
    match game.game_mode {
        GameMode::GridScreen | GameMode::CombatScreen => {
            game.paused_from = game.game_mode;
            game.game_mode = GameMode::PauseScreen;
        }
        GameMode::PauseScreen => game.game_mode = game.paused_from,
        GameMode::GuideScreen => game.game_mode = game.guide_return_to,
        GameMode::MultiplayerScreen => {
            game.game_mode = GameMode::TitleScreen;
            game.lobby_selected = None;
        }
        _ => {}
    }
}

pub fn update(game: &mut Game, rl: &RaylibHandle, hud: &HudRects, mouse_position: Vector2) -> bool {
    if game.game_mode != GameMode::PauseScreen {
        return false;
    }
    let clicked = input::mouse_is_clicked(rl);
    if input::is_button_clicked(mouse_position, clicked, hud.btn_pause_exit) {
        return true;
    }
    if input::is_button_clicked(mouse_position, clicked, hud.btn_pause_settings) {
        // TODO: écran settings
    }
    if input::is_button_clicked(mouse_position, clicked, hud.btn_pause_guide) {
        game.guide_return_to = GameMode::PauseScreen;
        game.game_mode = GameMode::GuideScreen;
        game.guide_scroll = 0.0;
    }
    if input::is_button_clicked(mouse_position, clicked, hud.btn_pause_play) {
        game.game_mode = game.paused_from;
    }
    if input::is_button_clicked(mouse_position, clicked, hud.btn_pause_back) {
        game.reset(rl, GameMode::TitleScreen);
    }
    false
}
