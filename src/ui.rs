use crate::assets::Assets;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
use raylib::prelude::*;

/// positions des boutons / bannières du HUD
pub struct HudRects {
    pub btn_end_turn: Rectangle,
    pub btn_wait: Rectangle,
    pub btn_speed: Rectangle,
    pub banner_your_turn: Rectangle,
    pub banner_enemy_turn: Rectangle,
    pub btn_retry: Rectangle,
    pub btn_back: Rectangle,
    pub btn_exit: Rectangle,
    pub btn_play_title_screen: Rectangle,
    pub btn_multi_title_screen: Rectangle,
    pub btn_settings_title_screen: Rectangle,
    pub btn_learn_title_screen: Rectangle,
    pub btn_exit_title_screen: Rectangle,
    pub btn_pause_play: Rectangle,
    pub btn_pause_guide: Rectangle,
    pub btn_pause_settings: Rectangle,
    pub btn_pause_back: Rectangle,
    pub btn_pause_exit: Rectangle,
    pub btn_guide_back: Rectangle,
    pub btn_mp_back: Rectangle,
    pub btn_mp_refresh: Rectangle,
    pub btn_mp_host: Rectangle,
    pub btn_mp_join: Rectangle,
}

// boutons assez larges pour les libellés français (« FIN DU TOUR », « PARAMÈTRES »)
const MENU_BTN_WIDTH: f32 = 220.0;
const MENU_BTN_HEIGHT: f32 = 52.0;
const WOOD_BTN_WIDTH: f32 = 200.0;
const WOOD_BTN_HEIGHT: f32 = 48.0;

impl HudRects {
    pub fn new() -> HudRects {
        let btn_end_turn = Rectangle {
            x: (SCREEN_WIDTH as f32) - WOOD_BTN_WIDTH - 20.0,
            y: (SCREEN_HEIGHT as f32) - WOOD_BTN_HEIGHT - 20.0,
            width: WOOD_BTN_WIDTH,
            height: WOOD_BTN_HEIGHT,
        };
        let btn_wait = Rectangle {
            y: btn_end_turn.y - WOOD_BTN_HEIGHT - 10.0,
            ..btn_end_turn
        };
        let banner_your_turn = Rectangle {
            x: (SCREEN_WIDTH as f32) - 384.0 - 20.0,
            y: 10.0,
            width: 384.0,
            height: 64.0,
        };
        let banner_enemy_turn = banner_your_turn;
        // à gauche de la bannière de tour, le seul coin toujours libre sur la grille
        let btn_speed = Rectangle {
            x: 20.0,
            y: banner_your_turn.y + 5.0,
            width: 80.0,
            height: 40.0,
        };
        let btn_retry = Rectangle {
            x: SCREEN_WIDTH as f32 / 2.0 - WOOD_BTN_WIDTH / 2.0, // centré
            y: SCREEN_HEIGHT as f32 / 2.0 + 60.0,                 // sous le texte
            width: WOOD_BTN_WIDTH,
            height: WOOD_BTN_HEIGHT,
        };
        let btn_back = Rectangle {
            y: btn_retry.y + WOOD_BTN_HEIGHT + 10.0,
            ..btn_retry
        };
        let btn_exit = Rectangle {
            y: btn_back.y + WOOD_BTN_HEIGHT + 10.0,
            ..btn_retry
        };
        // title screen ------------------------------------------------------
        let menu_btn_gap = 16.0;
        let menu_total_height = MENU_BTN_HEIGHT * 5.0 + menu_btn_gap * 4.0;
        let menu_top = SCREEN_HEIGHT as f32 / 2.0 - menu_total_height / 2.0 + 60.0;
        let title_button = |i: f32| Rectangle {
            x: SCREEN_WIDTH as f32 / 2.0 - MENU_BTN_WIDTH / 2.0,
            y: menu_top + i * (MENU_BTN_HEIGHT + menu_btn_gap),
            width: MENU_BTN_WIDTH,
            height: MENU_BTN_HEIGHT,
        };

        // menu pause : mêmes boutons que la fin de partie ------------------------
        let pause_total_height = WOOD_BTN_HEIGHT * 5.0 + menu_btn_gap * 4.0;
        let pause_top = SCREEN_HEIGHT as f32 / 2.0 - pause_total_height / 2.0;
        let pause_button = |i: f32| Rectangle {
            x: SCREEN_WIDTH as f32 / 2.0 - WOOD_BTN_WIDTH / 2.0,
            y: pause_top + i * (WOOD_BTN_HEIGHT + menu_btn_gap),
            width: WOOD_BTN_WIDTH,
            height: WOOD_BTN_HEIGHT,
        };
        // Back des écrans Guide / Settings / entraînement : en bas au centre
        let btn_guide_back = Rectangle {
            x: SCREEN_WIDTH as f32 / 2.0 - WOOD_BTN_WIDTH / 2.0,
            y: SCREEN_HEIGHT as f32 - WOOD_BTN_HEIGHT - 20.0,
            width: WOOD_BTN_WIDTH,
            height: WOOD_BTN_HEIGHT,
        };

        // écran Multiplayer : Back à gauche, Host / Join à droite, Refresh dans le coin
        // haut droit du panneau
        let mp_buttons_y = SCREEN_HEIGHT as f32 - WOOD_BTN_HEIGHT - 30.0;
        let btn_mp_back = Rectangle {
            x: 60.0,
            y: mp_buttons_y,
            width: 180.0,
            height: WOOD_BTN_HEIGHT,
        };
        let btn_mp_join = Rectangle {
            x: SCREEN_WIDTH as f32 - 60.0 - 220.0,
            y: mp_buttons_y,
            width: 220.0,
            height: WOOD_BTN_HEIGHT,
        };
        let btn_mp_host = Rectangle {
            x: btn_mp_join.x - 16.0 - 220.0,
            ..btn_mp_join
        };
        let btn_mp_refresh = Rectangle {
            x: SCREEN_WIDTH as f32 - 60.0 - 24.0 - 170.0,
            y: 114.0,
            width: 170.0,
            height: WOOD_BTN_HEIGHT,
        };

        HudRects {
            btn_end_turn,
            btn_wait,
            btn_speed,
            banner_your_turn,
            banner_enemy_turn,
            btn_retry,
            btn_back,
            btn_exit,
            btn_play_title_screen: title_button(0.0),
            btn_multi_title_screen: title_button(1.0),
            btn_settings_title_screen: title_button(2.0),
            btn_learn_title_screen: title_button(3.0),
            btn_exit_title_screen: title_button(4.0),
            btn_pause_play: pause_button(0.0),
            btn_pause_guide: pause_button(1.0),
            btn_pause_settings: pause_button(2.0),
            btn_pause_back: pause_button(3.0),
            btn_pause_exit: pause_button(4.0),
            btn_guide_back,
            btn_mp_back,
            btn_mp_refresh,
            btn_mp_host,
            btn_mp_join,
        }
    }
}

// couleurs communes des nouveaux écrans ------------------------------------------
pub const CREAM: Color = Color::new(236, 224, 192, 255);
pub const GOLD: Color = Color::new(240, 196, 70, 255);
pub const MUTED: Color = Color::new(165, 160, 150, 255);
pub const GOOD: Color = Color::new(120, 230, 120, 255);
pub const BAD: Color = Color::new(255, 110, 110, 255);
pub const PANEL: Color = Color::new(18, 16, 20, 225);
pub const PANEL_LINE: Color = Color::new(255, 255, 255, 60);

/// style de bouton : gris foncé des menus, ou bois du jeu
#[derive(Clone, Copy, PartialEq)]
pub enum Skin {
    Menu,
    Wood,
}

/// bouton dessiné avec le fond de l'art original, étiré à la largeur de `rect`, et son
/// texte dans la langue choisie. Survol = texte doré, désactivé = grisé
pub fn draw_button(
    d: &mut RaylibDrawHandle,
    assets: &Assets,
    rect: Rectangle,
    label: &str,
    skin: Skin,
    mouse: Vector2,
    enabled: bool,
) {
    let texture = match skin {
        Skin::Menu => &assets.btn_menu_base,
        Skin::Wood => &assets.btn_wood_base,
    };
    let hovered = enabled && rect.check_collision_point_rec(mouse);
    let tint = if enabled {
        Color::WHITE
    } else {
        Color::new(120, 120, 120, 220)
    };
    draw_stretched(d, texture, rect, 14, tint);
    if hovered {
        d.draw_rectangle_rec(
            Rectangle::new(rect.x + 6.0, rect.y + 6.0, rect.width - 12.0, rect.height - 12.0),
            Color::new(255, 230, 170, 22),
        );
    }
    let color = if !enabled {
        Color::new(150, 145, 135, 255)
    } else if hovered {
        Color::new(255, 214, 120, 255)
    } else {
        CREAM
    };
    let base_size = rect.height * 0.55;
    pixel_label(d, &assets.alert_font, rect, &label.to_uppercase(), base_size, color);
}

/// bannière de tour (ruban), texte centré
pub fn draw_banner(d: &mut RaylibDrawHandle, assets: &Assets, rect: Rectangle, label: &str, enemy: bool) {
    let texture = if enemy {
        &assets.banner_enemy_base
    } else {
        &assets.banner_turn_base
    };
    draw_stretched(d, texture, rect, 40, Color::WHITE);
    let inner = Rectangle::new(rect.x + 40.0, rect.y, rect.width - 80.0, rect.height - 4.0);
    pixel_label(d, &assets.alert_font, inner, &label.to_uppercase(), 40.0, CREAM);
}

/// texture étirée en largeur : les `edge` premiers et derniers pixels (coins décorés)
/// restent intacts, le milieu (uni) s'étire
fn draw_stretched(d: &mut RaylibDrawHandle, texture: &Texture2D, rect: Rectangle, edge: i32, tint: Color) {
    let info = raylib::ffi::NPatchInfo {
        source: Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32),
        left: edge,
        top: 0,
        right: edge,
        bottom: 0,
        layout: raylib::ffi::NPatchLayout::NPATCH_THREE_PATCH_HORIZONTAL as i32,
    };
    d.draw_texture_n_patch(texture, info, rect, Vector2::zero(), 0.0, tint);
}

/// texte pixel centré dans `rect` avec une ombre, rétréci s'il ne rentre pas
fn pixel_label(d: &mut RaylibDrawHandle, font: &Font, rect: Rectangle, text: &str, size: f32, color: Color) {
    let max_width = rect.width - 28.0;
    let mut size = size;
    let mut measure = font.measure_text(text, size, 2.0);
    while measure.x > max_width && size > 12.0 {
        size -= 1.0;
        measure = font.measure_text(text, size, 2.0);
    }
    let pos = Vector2::new(
        (rect.x + rect.width / 2.0 - measure.x / 2.0).round(),
        (rect.y + rect.height / 2.0 - measure.y / 2.0).round(),
    );
    d.draw_text_ex(font, text, pos + Vector2::new(0.0, 3.0), size, 2.0, Color::new(8, 7, 10, 200));
    d.draw_text_ex(font, text, pos, size, 2.0, color);
}

/// titre d'écran centré, avec une ombre pour rester lisible sur les fonds clairs
pub fn draw_title(d: &mut impl RaylibDraw, font: &Font, text: &str, center_x: f32, y: f32, size: f32) {
    let measure = font.measure_text(text, size, 2.0);
    let pos = Vector2::new(center_x - measure.x / 2.0, y);
    d.draw_text_ex(font, text, pos + Vector2::new(2.0, 3.0), size, 2.0, Color::new(0, 0, 0, 170));
    d.draw_text_ex(font, text, pos, size, 2.0, Color::WHITE);
}

/// touche de clavier dessinée comme une petite touche. Renvoie sa largeur
pub fn draw_keycap(d: &mut impl RaylibDraw, font: &Font, pos: Vector2, label: &str, size: f32) -> f32 {
    let measure = font.measure_text(label, size, 1.0);
    let width = (measure.x + size * 0.9).max(size * 1.6);
    let height = size * 1.45;
    let rect = Rectangle::new(pos.x, pos.y, width, height);
    // dessous plus sombre = relief de la touche
    d.draw_rectangle_rounded(
        Rectangle::new(rect.x, rect.y + 3.0, rect.width, rect.height),
        0.3,
        6,
        Color::new(20, 18, 16, 255),
    );
    d.draw_rectangle_rounded(rect, 0.3, 6, Color::new(232, 224, 205, 255));
    d.draw_rectangle_rounded_lines_ex(rect, 0.3, 6, 1.5, Color::new(120, 110, 95, 255));
    d.draw_text_ex(
        font,
        label,
        Vector2::new(
            rect.x + width / 2.0 - measure.x / 2.0,
            rect.y + height / 2.0 - measure.y / 2.0,
        ),
        size,
        1.0,
        Color::new(30, 26, 22, 255),
    );
    width
}

/// pastille arrondie avec du texte (facteurs de duel, badges). Renvoie sa largeur
pub fn draw_chip(
    d: &mut impl RaylibDraw,
    font: &Font,
    pos: Vector2,
    text: &str,
    size: f32,
    color: Color,
) -> f32 {
    let measure = font.measure_text(text, size, 1.0);
    let rect = Rectangle::new(pos.x, pos.y, measure.x + size * 1.1, measure.y + size * 0.45);
    d.draw_rectangle_rounded(rect, 0.6, 8, Color::new(color.r / 6, color.g / 6, color.b / 6, 235));
    d.draw_rectangle_rounded_lines_ex(rect, 0.6, 8, 1.5, Color::new(color.r, color.g, color.b, 170));
    d.draw_text_ex(
        font,
        text,
        Vector2::new(rect.x + size * 0.55, rect.y + size * 0.22),
        size,
        1.0,
        color,
    );
    rect.width
}

/// interrupteur à deux choix ou plus : l'option active est dorée. Renvoie les rectangles
/// des options (pour les clics)
pub fn draw_segmented(
    d: &mut RaylibDrawHandle,
    font: &Font,
    rect: Rectangle,
    options: &[&str],
    active: usize,
    mouse: Vector2,
) {
    for (i, (option, cell)) in options.iter().zip(segment_rects(rect, options.len())).enumerate() {
        let is_active = i == active;
        let hovered = cell.check_collision_point_rec(mouse);
        let background = if is_active {
            Color::new(92, 70, 28, 240)
        } else if hovered {
            Color::new(52, 48, 50, 240)
        } else {
            Color::new(32, 30, 34, 240)
        };
        d.draw_rectangle_rec(cell, background);
        d.draw_rectangle_lines_ex(cell, if is_active { 2.0 } else { 1.0 }, if is_active { GOLD } else { PANEL_LINE });
        let size = 20.0;
        let measure = font.measure_text(option, size, 1.0);
        d.draw_text_ex(
            font,
            option,
            Vector2::new(
                cell.x + cell.width / 2.0 - measure.x / 2.0,
                cell.y + cell.height / 2.0 - measure.y / 2.0,
            ),
            size,
            1.0,
            if is_active { Color::WHITE } else { MUTED },
        );
    }
}

/// découpe d'un interrupteur en cases égales (dessin et clics partagent ce calcul)
pub fn segment_rects(rect: Rectangle, count: usize) -> Vec<Rectangle> {
    let width = rect.width / count as f32;
    (0..count)
        .map(|i| Rectangle::new(rect.x + i as f32 * width, rect.y, width, rect.height))
        .collect()
}

/// panneau sombre aux coins arrondis
pub fn draw_panel(d: &mut impl RaylibDraw, rect: Rectangle) {
    d.draw_rectangle_rounded(rect, 0.04, 8, PANEL);
    d.draw_rectangle_rounded_lines_ex(rect, 0.04, 8, 2.0, PANEL_LINE);
}

pub fn draw_text_button(d: &mut RaylibDrawHandle, font: &Font, rect: Rectangle, label: &str) {
    d.draw_rectangle_rec(rect, Color::new(30, 30, 30, 220));
    d.draw_rectangle_lines_ex(rect, 2.0, Color::WHITE);
    let font_size = 22.0;
    let text_size = font.measure_text(label, font_size, 1.0);
    d.draw_text_ex(
        font,
        label,
        Vector2::new(
            rect.x + rect.width / 2.0 - text_size.x / 2.0,
            rect.y + rect.height / 2.0 - text_size.y / 2.0,
        ),
        font_size,
        1.0,
        Color::WHITE,
    );
}

pub fn draw_health_bar(
    d: &mut RaylibDrawHandle,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    hp_points: i32,
    max_hp_points: i32,
) {
    let ratio = if max_hp_points > 0 {
        hp_points as f32 / max_hp_points as f32
    } else {
        0.0
    };
    let ratio = ratio.clamp(0.0, 1.0);

    // fond (barre vide)
    d.draw_rectangle(
        x as i32,
        y as i32,
        width as i32,
        height as i32,
        Color::new(60, 60, 60, 255),
    );

    // remplissage (HP actuels)
    let fill_width = (width * ratio) as i32;
    d.draw_rectangle(x as i32, y as i32, fill_width, height as i32, Color::LIME);

    // contour
    d.draw_rectangle_lines(
        x as i32,
        y as i32,
        width as i32,
        height as i32,
        Color::WHITE,
    );
}

/// texte coupé en lignes pour tenir dans `width`. Renvoie la hauteur utilisée
pub fn draw_wrapped(
    d: &mut impl RaylibDraw,
    font: &Font,
    text: &str,
    pos: Vector2,
    width: f32,
    size: f32,
    color: Color,
) -> f32 {
    let line_height = size * 1.3;
    let mut y = pos.y;
    for line in wrap(font, text, width, size) {
        d.draw_text_ex(font, &line, Vector2::new(pos.x, y), size, 1.0, color);
        y += line_height;
    }
    y - pos.y
}

/// découpe `text` en lignes qui tiennent dans `width` (coupure aux espaces)
pub fn wrap(font: &Font, text: &str, width: f32, size: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split(' ') {
        // « : », « ! », « ? » isolés (typo française) restent sur la ligne du mot d'avant
        if matches!(word, ":" | "!" | "?" | ";") && !current.is_empty() {
            current = format!("{current} {word}");
            continue;
        }
        if word.is_empty() && current.is_empty() {
            continue;
        }
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if !current.is_empty() && font.measure_text(&candidate, size, 1.0).x > width {
            lines.push(std::mem::replace(&mut current, word.to_string()));
        } else {
            current = candidate;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}
