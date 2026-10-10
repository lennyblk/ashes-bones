use raylib::consts::KeyboardKey::*;
use raylib::prelude::*;

use crate::game::Game;
use crate::game_mode::GameMode;
use crate::input;
use crate::ui::HudRects;
use crate::unit::Faction;

// mise en page (partagée entre update pour les clics et render pour le dessin)
pub const PANEL: Rectangle = Rectangle {
    x: 300.0,
    y: 110.0,
    width: 600.0,
    height: 520.0,
};
const GAME_NAME_MAX: usize = 24;
const PLAYER_NAME_MAX: usize = 16;

#[derive(Clone, Copy, PartialEq)]
pub enum HostField {
    GameName,
    PlayerName,
}

/// formulaire de l'écran Host game, gardé d'une partie à l'autre
pub struct HostForm {
    pub game_name: String,
    pub player_name: String,
    pub faction: Faction,
    // champ où l'on écrit
    pub focus: HostField,
}

impl HostForm {
    pub fn new() -> HostForm {
        HostForm {
            game_name: String::new(),
            player_name: String::new(),
            faction: Faction::Human,
            focus: HostField::GameName,
        }
    }

    /// les deux noms sont remplis : on peut héberger
    pub fn is_complete(&self) -> bool {
        !self.game_name.trim().is_empty() && !self.player_name.trim().is_empty()
    }
}

pub fn field_rect(field: HostField) -> Rectangle {
    let y = match field {
        HostField::GameName => PANEL.y + 70.0,
        HostField::PlayerName => PANEL.y + 180.0,
    };
    Rectangle::new(PANEL.x + 30.0, y, PANEL.width - 60.0, 48.0)
}

pub fn faction_rect(faction: Faction) -> Rectangle {
    let width = (PANEL.width - 60.0 - 20.0) / 2.0;
    let x = match faction {
        Faction::Human => PANEL.x + 30.0,
        Faction::Undead => PANEL.x + 30.0 + width + 20.0,
    };
    Rectangle::new(x, PANEL.y + 300.0, width, 180.0)
}

pub fn update(
    game: &mut Game,
    rl: &mut RaylibHandle,
    hud: &HudRects,
    mouse_position: Vector2,
    click_consumed: bool,
) {
    if game.game_mode != GameMode::HostScreen {
        return;
    }
    let clicked = input::mouse_is_clicked(rl) && !click_consumed;
    if input::is_button_clicked(mouse_position, clicked, hud.btn_mp_back) {
        game.game_mode = GameMode::MultiplayerScreen;
        return;
    }
    let form = &mut game.host_form;
    // clic : le champ où l'on écrit, ou la faction
    for field in [HostField::GameName, HostField::PlayerName] {
        if input::is_button_clicked(mouse_position, clicked, field_rect(field)) {
            form.focus = field;
        }
    }
    for faction in [Faction::Human, Faction::Undead] {
        if input::is_button_clicked(mouse_position, clicked, faction_rect(faction)) {
            form.faction = faction;
        }
    }
    if rl.is_key_pressed(KEY_TAB) {
        form.focus = match form.focus {
            HostField::GameName => HostField::PlayerName,
            HostField::PlayerName => HostField::GameName,
        };
    }

    let (text, max_len) = match form.focus {
        HostField::GameName => (&mut form.game_name, GAME_NAME_MAX),
        HostField::PlayerName => (&mut form.player_name, PLAYER_NAME_MAX),
    };
    // caractères tapés depuis la dernière frame (plusieurs si on tape vite)
    while let Some(c) = rl.get_char_pressed() {
        // '|' sépare les champs des annonces sur le réseau : interdit dans les noms
        if c != '|' && !c.is_control() && text.chars().count() < max_len {
            text.push(c);
        }
    }
    if rl.is_key_pressed(KEY_BACKSPACE) || rl.is_key_pressed_repeat(KEY_BACKSPACE) {
        text.pop();
    }
    // Host game : ouvre la partie (5d)
}
