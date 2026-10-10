use std::net::IpAddr;

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

/// ce que la liste demande à main.rs, qui possède le réseau
pub enum LobbyAction {
    Join(IpAddr),
    Cancel,
}

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
    connecting: bool,
) -> Option<LobbyAction> {
    if game.game_mode != GameMode::MultiplayerScreen {
        return None;
    }
    let clicked = input::mouse_is_clicked(rl) && !click_consumed;
    // connexion en cours : la liste est figée, Back devient Cancel
    if connecting {
        let cancel = input::is_button_clicked(mouse_position, clicked, hud.btn_mp_back);
        return cancel.then_some(LobbyAction::Cancel);
    }
    if input::is_button_clicked(mouse_position, clicked, hud.btn_mp_back) {
        game.game_mode = GameMode::TitleScreen;
        game.lobby_selected = None;
        game.lobby_error = None;
        return None;
    }
    let Some(browser) = browser else {
        return None; // port déjà pris : rien à lister
    };
    if input::is_button_clicked(mouse_position, clicked, hud.btn_mp_refresh) {
        browser.clear();
        game.lobby_selected = None;
    }
    // clic sur une partie ouverte : elle devient la sélection (FULL ou autre version : non)
    for (i, lan_game) in browser.games().iter().take(MAX_ROWS).enumerate() {
        if lan_game.info.is_joinable()
            && input::is_button_clicked(mouse_position, clicked, row_rect(i))
        {
            game.lobby_selected = Some(lan_game.address);
        }
    }
    // la partie choisie a disparu ou n'est plus rejoignable : on l'oublie
    if let Some(ip) = game.lobby_selected {
        if !browser
            .games()
            .iter()
            .any(|g| g.address == ip && g.info.is_joinable())
        {
            game.lobby_selected = None;
        }
    }
    if input::is_button_clicked(mouse_position, clicked, hud.btn_mp_host) {
        game.game_mode = GameMode::HostScreen;
    }
    if input::is_button_clicked(mouse_position, clicked, hud.btn_mp_join) {
        return game.lobby_selected.map(LobbyAction::Join);
    }
    None
}
