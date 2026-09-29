use raylib::prelude::*;

use crate::SCREEN_WIDTH;
use crate::game::Game;
use crate::game_mode::GameMode;
use crate::input;
use crate::unit::Faction;

pub fn update(game: &mut Game, rl: &RaylibHandle, mouse_position: Vector2, click_consumed: bool) {
    if game.game_mode != GameMode::FactionSelectionScreen {
        return;
    }
    if input::cancel_pressed(rl) {
        game.game_mode = GameMode::TitleScreen;
    } else if !click_consumed && input::mouse_is_clicked(rl) {
        game.player_faction = if mouse_position.x < SCREEN_WIDTH as f32 / 2.0 {
            Faction::Human
        } else {
            Faction::Undead
        };
        game.game_mode = GameMode::GridScreen;
    }
}
