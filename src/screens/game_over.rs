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
    if !matches!(game.game_mode, GameMode::Victory | GameMode::Defeat) {
        return false;
    }
    let clicked = input::mouse_is_clicked(rl);
    if input::is_button_clicked(mouse_position, clicked, hud.btn_exit) {
        return true;
    }
    if input::is_button_clicked(mouse_position, clicked, hud.btn_back) {
        game.reset(rl, GameMode::TitleScreen);
        *click_consumed = true;
    }
    if input::is_button_clicked(mouse_position, clicked, hud.btn_retry) {
        game.reset(rl, GameMode::GridScreen);
        *click_consumed = true;
    }
    false
}
