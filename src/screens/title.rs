use raylib::prelude::*;

use crate::game::Game;
use crate::game_mode::GameMode;
use crate::input;
use crate::ui::HudRects;

pub fn update(
    game: &mut Game,
    rl: &RaylibHandle,
    hud: &HudRects,
    mouse_position: Vector2,
    click_consumed: &mut bool,
) -> bool {
    if game.game_mode != GameMode::TitleScreen {
        return false;
    }
    let clicked = input::mouse_is_clicked(rl);
    if input::is_button_clicked(mouse_position, clicked, hud.btn_exit_title_screen) {
        return true;
    }
    if input::is_button_clicked(mouse_position, clicked, hud.btn_multi_title_screen) {
        // TODO: mode multijoueur
    }
    if input::is_button_clicked(mouse_position, clicked, hud.btn_settings_title_screen) {
        // TODO: écran settings
    }
    if input::is_button_clicked(mouse_position, clicked, hud.btn_learn_title_screen) {
        game.guide_return_to = GameMode::TitleScreen;
        game.game_mode = GameMode::GuideScreen;
        game.guide_scroll = 0.0;
        *click_consumed = true;
    }
    if input::is_button_clicked(mouse_position, clicked, hud.btn_play_title_screen) {
        game.game_mode = GameMode::FactionSelectionScreen;
        *click_consumed = true;
    }
    false
}
