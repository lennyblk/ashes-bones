//! écran Settings : langue, confort, et touches (clic sur une touche puis appui sur la
//! nouvelle). Tout est enregistré dès qu'on change quelque chose

use raylib::prelude::*;

use crate::game::Game;
use crate::game_mode::GameMode;
use crate::input;
use crate::settings::{self, Bind, Lang, Settings, tr};
use crate::ui::{self, HudRects};

// mise en page (partagée entre update pour les clics et render pour le dessin)
pub const GENERAL: Rectangle = Rectangle {
    x: 40.0,
    y: 84.0,
    width: 540.0,
    height: 592.0,
};
pub const CONTROLS: Rectangle = Rectangle {
    x: 600.0,
    y: 84.0,
    width: 560.0,
    height: 592.0,
};
const GENERAL_ROW: f32 = 96.0;
pub const KEY_ROW: f32 = 27.0;

/// une ligne de la colonne de gauche : titre, explication, et ses choix
pub struct Toggle {
    pub title: &'static str,
    pub detail: &'static str,
    pub options: [&'static str; 2],
    pub active: usize,
}

pub fn toggles(s: &Settings) -> [Toggle; 5] {
    let on_off = [tr("Off", "Non"), tr("On", "Oui")];
    [
        Toggle {
            title: tr("Language", "Langue"),
            detail: tr("Menus, guide and combat texts", "Menus, guide et textes de combat"),
            options: ["English", "Français"],
            active: (s.lang == Lang::Fr) as usize,
        },
        Toggle {
            title: tr("Map speed at start", "Vitesse de la carte au départ"),
            detail: tr("x2 speeds up moves (not duels)", "x2 accélère les déplacements (pas les duels)"),
            options: ["x1", "x2"],
            active: s.fast_map as usize,
        },
        Toggle {
            title: tr("Grid lines", "Lignes de la grille"),
            detail: tr("Show the tiles of the map", "Affiche les cases de la carte"),
            options: on_off,
            active: s.grid_lines as usize,
        },
        Toggle {
            title: tr("Attack preview", "Aperçu d'attaque"),
            detail: tr("Damage forecast when you hover an enemy", "Dégâts prévus au survol d'un ennemi"),
            options: on_off,
            active: s.attack_preview as usize,
        },
        Toggle {
            title: tr("Tile bonuses", "Bonus des cases"),
            detail: tr("+12% style badges on attack tiles", "Badges +12% sur les cases d'attaque"),
            options: on_off,
            active: s.tile_bonuses as usize,
        },
    ]
}

pub fn toggle_title_pos(i: usize) -> Vector2 {
    Vector2::new(GENERAL.x + 28.0, GENERAL.y + 70.0 + i as f32 * GENERAL_ROW)
}

pub fn toggle_rect(i: usize) -> Rectangle {
    let title = toggle_title_pos(i);
    Rectangle::new(title.x, title.y + 52.0, 300.0, 34.0)
}

/// rangée de chaque touche réglable : (action, y). Carte puis duel, avec la place des titres
pub fn key_rows() -> Vec<(Bind, f32)> {
    let mut rows = Vec::new();
    let mut y = CONTROLS.y + 94.0 + 2.0 * KEY_ROW;
    for bind in Bind::MAP {
        rows.push((bind, y));
        y += KEY_ROW;
    }
    y += 46.0 + KEY_ROW;
    for bind in Bind::DUEL {
        rows.push((bind, y));
        y += KEY_ROW;
    }
    rows
}

/// titre du groupe Carte / Duel
pub fn map_heading_y() -> f32 {
    CONTROLS.y + 64.0
}

pub fn duel_heading_y() -> f32 {
    key_rows()[Bind::MAP.len() - 1].1 + KEY_ROW + 16.0
}

/// la touche, cliquable, à droite de sa ligne
pub fn key_rect(y: f32) -> Rectangle {
    Rectangle::new(CONTROLS.x + CONTROLS.width - 200.0, y - 1.0, 170.0, KEY_ROW - 3.0)
}

pub fn reset_rect() -> Rectangle {
    Rectangle::new(
        CONTROLS.x + CONTROLS.width - 210.0,
        CONTROLS.y + 14.0,
        190.0,
        40.0,
    )
}

pub fn open(game: &mut Game, from: GameMode) {
    game.settings_return_to = from;
    game.settings_capture = None;
    game.game_mode = GameMode::SettingsScreen;
}

pub fn update(game: &mut Game, rl: &mut RaylibHandle, hud: &HudRects, mouse: Vector2, click_consumed: bool) {
    if game.game_mode != GameMode::SettingsScreen {
        return;
    }
    let clicked = input::mouse_is_clicked(rl) && !click_consumed;
    let mut s = settings::get();

    // on attend la nouvelle touche : Échap ou un clic annulent
    if let Some(bind) = game.settings_capture {
        if let Some(key) = rl.get_key_pressed() {
            if key != KeyboardKey::KEY_ESCAPE {
                s.rebind(bind, key);
                settings::set(s);
            }
            game.settings_capture = None;
        } else if clicked {
            game.settings_capture = None;
        }
        return;
    }

    if input::is_button_clicked(mouse, clicked, hud.btn_guide_back) {
        game.game_mode = game.settings_return_to;
        return;
    }
    if !clicked {
        return;
    }
    for i in 0..toggles(&s).len() {
        for (option, cell) in ui::segment_rects(toggle_rect(i), 2).iter().enumerate() {
            if !cell.check_collision_point_rec(mouse) {
                continue;
            }
            let on = option == 1;
            match i {
                0 => s.lang = if on { Lang::Fr } else { Lang::En },
                1 => {
                    s.fast_map = on;
                    game.grid_speed = if on { 2.0 } else { 1.0 };
                }
                2 => s.grid_lines = on,
                3 => s.attack_preview = on,
                _ => s.tile_bonuses = on,
            }
        }
    }
    for (bind, y) in key_rows() {
        if key_rect(y).check_collision_point_rec(mouse) {
            game.settings_capture = Some(bind);
        }
    }
    if reset_rect().check_collision_point_rec(mouse) {
        s.reset_keys();
    }
    if s != settings::get() {
        settings::set(s);
    }
}
