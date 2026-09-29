use raylib::prelude::*;

use crate::game::Game;
use crate::game_mode::GameMode;
use crate::input;
use crate::ui::HudRects;

pub fn update(game: &mut Game, rl: &RaylibHandle, hud: &HudRects, mouse_position: Vector2) {
    if game.game_mode == GameMode::GuideScreen
        && input::is_button_clicked(
            mouse_position,
            input::mouse_is_clicked(rl),
            hud.btn_guide_back,
        )
    {
        game.game_mode = game.guide_return_to;
    }
}
