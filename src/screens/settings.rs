use raylib::prelude::*;

use crate::SCREEN_WIDTH;
use crate::game::Game;
use crate::game_mode::GameMode;
use crate::input;
use crate::network::{DISCOVERY_PORT, PORT, VERSION};
use crate::ui::HudRects;

pub const HEADER_HEIGHT: f32 = 90.0;
const SIDE_MARGIN: f32 = 40.0;
pub const PANEL_PAD: f32 = 32.0;
pub const COLUMN_GAP: f32 = 40.0;
pub const HEADING_SIZE: f32 = 30.0;
pub const TEXT_SIZE: f32 = 20.0;
pub const HEADING_HEIGHT: f32 = 48.0;
pub const ROW_HEIGHT: f32 = 34.0;
pub const SECTION_GAP: f32 = 24.0;
pub const KEY_COLUMN: f32 = 170.0;

pub struct Section {
    pub title: &'static str,
    pub keys: bool,
    pub rows: Vec<(&'static str, String)>,
}

fn section(title: &'static str, keys: bool, rows: &[(&'static str, &str)]) -> Section {
    Section {
        title,
        keys,
        rows: rows
            .iter()
            .map(|(key, text)| (*key, String::from(*text)))
            .collect(),
    }
}

pub fn columns() -> [Vec<Section>; 2] {
    [
        vec![
            section(
                "Controls",
                true,
                &[
                    ("Left click", "Select, move, attack, heal"),
                    ("B", "Cancel the current action"),
                    ("Esc", "Pause menu / go back"),
                    ("F", "Map speed x1 / x2"),
                ],
            ),
            section(
                "Minigames",
                true,
                &[
                    ("Left click", "Click inside the zone"),
                    ("Mouse", "Priest / Necro: stay in the aura"),
                    ("Arrows", "Assassin / Wraith sequence"),
                    ("WASD", "Same (ZQSD on AZERTY)"),
                ],
            ),
        ],
        vec![
            section(
                "Host game",
                true,
                &[
                    ("Tab", "Switch name field"),
                    ("Enter", "Open the game"),
                    ("Backspace", "Erase a letter"),
                ],
            ),
            Section {
                title: "About",
                keys: false,
                rows: vec![
                    ("Version", format!("v{VERSION}")),
                    ("Online", String::from("Same local network only")),
                    ("Ports", format!("TCP {PORT}, UDP {DISCOVERY_PORT}")),
                    ("Updates", String::from("Automatic, restart to apply")),
                ],
            },
        ],
    ]
}

pub fn panel_rect(hud: &HudRects) -> Rectangle {
    Rectangle {
        x: SIDE_MARGIN,
        y: HEADER_HEIGHT,
        width: SCREEN_WIDTH as f32 - SIDE_MARGIN * 2.0,
        height: hud.btn_guide_back.y - 12.0 - HEADER_HEIGHT,
    }
}

pub fn update(game: &mut Game, rl: &RaylibHandle, hud: &HudRects, mouse_position: Vector2) {
    if game.game_mode != GameMode::SettingsScreen {
        return;
    }
    if input::is_button_clicked(
        mouse_position,
        input::mouse_is_clicked(rl),
        hud.btn_guide_back,
    ) {
        game.game_mode = game.settings_return_to;
    }
}
