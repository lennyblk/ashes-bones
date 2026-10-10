use raylib::prelude::*;

use crate::game::Game;
use crate::game_mode::GameMode;
use crate::input;
use crate::network::Browser;
use crate::ui::HudRects;

// mise en page de la liste (partagée entre update pour les clics et render pour le dessin)
pub const PANEL: Rectangle = Rectangle {
    x: 60.0,
    y: 100.0,
    width: 1080.0,
    height: 540.0,
};
pub const MAX_ROWS: usize = 7;
const ROW_HEIGHT: f32 = 46.0;
const ROW_GAP: f32 = 4.0;
const ROWS_TOP: f32 = 210.0;

/// rectangle de la ligne `i` de la liste
pub fn row_rect(i: usize) -> Rectangle {
    Rectangle {
        x: PANEL.x + 24.0,
        y: ROWS_TOP + i as f32 * (ROW_HEIGHT + ROW_GAP),
        width: PANEL.width - 48.0,
        height: ROW_HEIGHT,
    }
}

pub fn update(
    game: &mut Game,
    rl: &RaylibHandle,
    hud: &HudRects,
    mouse_position: Vector2,
    browser: Option<&mut Browser>,
    click_consumed: bool,
) {
    if game.game_mode != GameMode::MultiplayerScreen {
        return;
    }
    let clicked = input::mouse_is_clicked(rl) && !click_consumed;
    if input::is_button_clicked(mouse_position, clicked, hud.btn_mp_back) {
        game.game_mode = GameMode::TitleScreen;
        game.lobby_selected = None;
        return;
    }
    let Some(browser) = browser else {
        return; // port déjà pris : rien à lister
    };
    if input::is_button_clicked(mouse_position, clicked, hud.btn_mp_refresh) {
        browser.clear();
        game.lobby_selected = None;
    }
    // clic sur une partie ouverte : elle devient la sélection (une FULL ne se rejoint pas)
    for (i, lan_game) in browser.games().iter().take(MAX_ROWS).enumerate() {
        if !lan_game.info.full && input::is_button_clicked(mouse_position, clicked, row_rect(i)) {
            game.lobby_selected = Some(lan_game.address);
        }
    }
    // la partie choisie a disparu ou est passée FULL : on l'oublie
    if let Some(ip) = game.lobby_selected {
        if !browser
            .games()
            .iter()
            .any(|g| g.address == ip && !g.info.full)
        {
            game.lobby_selected = None;
        }
    }
    // Host game -> écran de création (5c), Join game -> connexion à la partie choisie (5d)
}
