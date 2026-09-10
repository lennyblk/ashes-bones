use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
use raylib::prelude::*;

/// positions des boutons / bannières du HUD
pub struct HudRects {
    pub btn_end_turn: Rectangle,
    pub btn_wait: Rectangle,
    pub banner_your_turn: Rectangle,
    pub banner_enemy_turn: Rectangle,
    pub btn_retry: Rectangle,
    pub btn_back: Rectangle,
    pub btn_exit: Rectangle,
    pub title: Rectangle,
    pub btn_play_title_screen: Rectangle,
    pub btn_settings_title_screen: Rectangle,
    pub btn_exit_title_screen: Rectangle,
}

impl HudRects {
    pub fn new() -> HudRects {
        let btn_end_turn = Rectangle {
            x: (SCREEN_WIDTH as f32) - 160.0 - 20.0,
            y: (SCREEN_HEIGHT as f32) - 48.0 - 20.0,
            width: 160.0,
            height: 48.0,
        };
        let btn_wait = Rectangle {
            x: (SCREEN_WIDTH as f32) - 160.0 - 20.0,
            y: btn_end_turn.y - 48.0 - 10.0,
            width: 160.0,
            height: 48.0,
        };
        let banner_your_turn = Rectangle {
            x: (SCREEN_WIDTH as f32) - 384.0 - 20.0,
            y: 10.0,
            width: 384.0,
            height: 64.0,
        };
        let banner_enemy_turn = banner_your_turn;
        let btn_retry = Rectangle {
            x: SCREEN_WIDTH as f32 / 2.0 - 64.0,  // centré, largeur 128
            y: SCREEN_HEIGHT as f32 / 2.0 + 60.0, // sous le texte
            width: 128.0,
            height: 48.0,
        };
        let btn_back = Rectangle {
            x: btn_retry.x,
            y: btn_retry.y + 48.0 + 10.0,
            width: 128.0,
            height: 48.0,
        };
        let btn_exit = Rectangle {
            x: btn_retry.x,
            y: btn_back.y + 48.0 + 10.0,
            width: 128.0,
            height: 48.0,
        };
        // title screen ------------------------------------------------------
        let title = Rectangle {
            x: SCREEN_WIDTH as f32 / 2.0 - 256.0, // centré, largeur 512
            y: 80.0,
            width: 512.0,
            height: 128.0,
        };
        let menu_btn_width = 144.0;
        let menu_btn_height = 52.0;
        let menu_btn_gap = 16.0;

        let menu_total_height = menu_btn_height * 3.0 + menu_btn_gap * 2.0;
        let menu_top = SCREEN_HEIGHT as f32 / 2.0 - menu_total_height / 2.0 + 60.0;
        let btn_play_title_screen = Rectangle {
            x: SCREEN_WIDTH as f32 / 2.0 - menu_btn_width / 2.0,
            y: menu_top,
            width: menu_btn_width,
            height: menu_btn_height,
        };
        let btn_settings_title_screen = Rectangle {
            x: btn_play_title_screen.x,
            y: btn_play_title_screen.y + menu_btn_height + menu_btn_gap,
            width: menu_btn_width,
            height: menu_btn_height,
        };
        let btn_exit_title_screen = Rectangle {
            x: btn_play_title_screen.x,
            y: btn_settings_title_screen.y + menu_btn_height + menu_btn_gap,
            width: menu_btn_width,
            height: menu_btn_height,
        };
        HudRects {
            btn_end_turn,
            btn_wait,
            banner_your_turn,
            banner_enemy_turn,
            btn_retry,
            btn_back,
            btn_exit,
            title,
            btn_play_title_screen,
            btn_settings_title_screen,
            btn_exit_title_screen,
        }
    }
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
