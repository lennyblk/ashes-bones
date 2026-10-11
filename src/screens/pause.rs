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
        // Échap pendant qu'on attend une touche : annule seulement l'attente
        GameMode::SettingsScreen if game.settings_capture.is_some() => {
            game.settings_capture = None
        }
        GameMode::SettingsScreen => game.game_mode = game.settings_return_to,
        // pendant un essai : on l'arrête. Sinon retour au guide
        GameMode::PracticeScreen => {
            if !game.practice.stop() {
                game.game_mode = GameMode::GuideScreen;
            }
        }
        GameMode::HostScreen => game.game_mode = GameMode::MultiplayerScreen,
        GameMode::MultiplayerScreen => {
            game.game_mode = GameMode::TitleScreen;
            game.lobby_selected = None;
        }
        _ => {}
    }
}

pub fn update(
    game: &mut Game,
    rl: &RaylibHandle,
    hud: &HudRects,
    mouse_position: Vector2,
    click_consumed: &mut bool,
) -> bool {
    if game.game_mode != GameMode::PauseScreen {
        return false;
    }
    let clicked = input::mouse_is_clicked(rl);
    if input::is_button_clicked(mouse_position, clicked, hud.btn_pause_exit) {
        return true;
    }
    if input::is_button_clicked(mouse_position, clicked, hud.btn_pause_settings) {
        crate::screens::settings::open(game, GameMode::PauseScreen);
        *click_consumed = true;
        return false;
    }
    if input::is_button_clicked(mouse_position, clicked, hud.btn_pause_guide) {
        crate::screens::guide::open(game, GameMode::PauseScreen);
        *click_consumed = true;
        return false;
    }
    if input::is_button_clicked(mouse_position, clicked, hud.btn_pause_play) {
        game.game_mode = game.paused_from;
    }
    if input::is_button_clicked(mouse_position, clicked, hud.btn_pause_back) {
        game.reset(rl, GameMode::TitleScreen);
    }
    false
}
