use crate::SCREEN_HEIGHT;
use crate::ui::HudRects;
use raylib::prelude::*;

// petite fenêtre de stats du perso sélectionné (dessinée dans render.rs)
pub const SCALE: f32 = 1.6;
pub const WIDTH: f32 = 130.0 * SCALE;
pub const HEIGHT: f32 = 95.0 * SCALE;
const MARGIN: f32 = 10.0;

/// window fixe dans le coin en bas a gauche de l'écran (perso sélectionné)
pub fn panel_rect() -> Rectangle {
    Rectangle {
        x: MARGIN,
        y: SCREEN_HEIGHT as f32 - HEIGHT - MARGIN,
        width: WIDTH,
        height: HEIGHT,
    }
}

/// window de l'ennemi inspecté, en bas a droite mais à gauche des boutons
/// end turn / wait pour pas les cacher
pub fn enemy_panel_rect(hud: &HudRects) -> Rectangle {
    Rectangle {
        x: hud.btn_end_turn.x - MARGIN - WIDTH,
        y: SCREEN_HEIGHT as f32 - HEIGHT - MARGIN,
        width: WIDTH,
        height: HEIGHT,
    }
}
