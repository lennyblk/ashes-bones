//! visite de vérification de l'affichage (version de développement seulement) :
//!   ASHES_SHOTS=1 cargo run
//! passe par chaque écran, dans les deux langues, en prend une capture (shot_*.png dans
//! le dossier du jeu) puis ferme le jeu. Les réglages enregistrés ne sont pas touchés.

use raylib::prelude::*;

use crate::TILE_SIZE;
use crate::game::Game;
use crate::game_mode::GameMode;
use crate::minigame::MiniGameKind;
use crate::screens::practice::Preset;
use crate::settings::{self, Lang};
use crate::unit::{PendingAction, UnitState};

pub fn active() -> bool {
    std::env::var_os("ASHES_SHOTS").is_some()
}

type Setup = fn(&mut Game, &mut RaylibHandle);

const STEPS: [(&str, Setup); 12] = [
    ("title", |g, _| g.game_mode = GameMode::TitleScreen),
    ("settings", |g, rl| {
        g.game_mode = GameMode::SettingsScreen;
        rl.set_mouse_position(Vector2::new(1050.0, 300.0));
    }),
    ("guide_basics", |g, _| {
        g.game_mode = GameMode::GuideScreen;
        g.guide_page = crate::screens::guide::GuidePage::Basics;
    }),
    ("guide_minigames", |g, rl| {
        g.guide_page = crate::screens::guide::GuidePage::MiniGames;
        rl.set_mouse_position(Vector2::new(470.0, 300.0));
    }),
    ("guide_positioning", |g, _| g.guide_page = crate::screens::guide::GuidePage::Positioning),
    ("practice_setup", |g, rl| {
        g.practice.apply(Preset::Flank);
        g.practice.board[1][2] = crate::screens::practice::Cell::Ally;
        g.game_mode = GameMode::PracticeScreen;
        rl.set_mouse_position(Vector2::new(900.0, 640.0));
    }),
    ("practice_runes", |g, rl| {
        g.practice.apply(Preset::MiniGame(MiniGameKind::Runes));
        g.practice.phase = crate::screens::practice::Phase::Playing {
            minigame: crate::minigame::MiniGame::new(MiniGameKind::Runes, &g.practice.my_side(), rl),
            armed: true,
        };
    }),
    ("practice_result", |g, _| {
        g.practice.streak = 3;
        g.practice.phase = crate::screens::practice::Phase::Result {
            result: crate::minigame::TimingResult::Perfect,
        };
    }),
    ("grid_preview", |g, rl| grid_setup(g, rl, false)),
    ("grid_choosing", |g, rl| grid_setup(g, rl, true)),
    ("pause", |g, _| {
        g.paused_from = GameMode::GridScreen;
        g.game_mode = GameMode::PauseScreen;
    }),
    ("victory", |g, _| g.game_mode = GameMode::Victory),
];

/// une petite mêlée : soldat (0) qui vise un spectre (6) encerclé, gardé par un squelette
fn grid_setup(game: &mut Game, rl: &mut RaylibHandle, choosing: bool) {
    game.reset(rl, GameMode::GridScreen);
    game.player_faction = crate::unit::Faction::Human;
    let mut positions = game.unit_positions();
    let spots = [(0, (4, 6)), (1, (6, 5)), (2, (6, 7)), (3, (3, 6)), (6, (6, 6)), (10, (7, 6))];
    for (i, tile) in spots {
        positions[i] = tile;
    }
    // les autres unités à l'écart, loin de la scène
    for (i, p) in positions.iter_mut().enumerate() {
        if !spots.iter().any(|(j, _)| *j == i) && p.0 < 10 && p.1 > 2 && p.1 < 10 {
            *p = (p.0, 1);
        }
    }
    game.place_units(&positions);
    game.selected_unit = Some(0);
    let target = if choosing {
        game.units[0].state = UnitState::ChoosingPosition;
        game.units[0].pending_action = Some(PendingAction::Attack(6));
        (5, 7)
    } else {
        (6, 6)
    };
    rl.set_mouse_position(Vector2::new(
        (target.0 * TILE_SIZE + TILE_SIZE / 2) as f32,
        (target.1 * TILE_SIZE + TILE_SIZE / 2) as f32,
    ));
}

pub struct Tour {
    step: usize,
    frame: u32,
    lang: Lang,
}

impl Tour {
    pub fn new() -> Tour {
        Tour {
            step: 0,
            frame: 0,
            lang: Lang::En,
        }
    }

    /// avant chaque frame : prépare l'écran au début d'une étape
    pub fn before_frame(&mut self, game: &mut Game, rl: &mut RaylibHandle) {
        if self.frame == 0 {
            let mut s = settings::get();
            s.lang = self.lang;
            s.tile_bonuses = true;
            s.attack_preview = true;
            settings::set_unsaved(s);
            (STEPS[self.step].1)(game, rl);
        }
        // la souris est replacée à chaque frame (la vraie souris pourrait bouger)
        if matches!(STEPS[self.step].0, "grid_preview" | "grid_choosing") {
            let target = if STEPS[self.step].0 == "grid_choosing" { (5, 7) } else { (6, 6) };
            rl.set_mouse_position(Vector2::new(
                (target.0 * TILE_SIZE + TILE_SIZE / 2) as f32,
                (target.1 * TILE_SIZE + TILE_SIZE / 2) as f32,
            ));
        }
    }

    /// après le dessin : capture au bout de quelques frames, puis étape suivante.
    /// Renvoie false quand la visite est finie
    pub fn after_frame(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) -> bool {
        self.frame += 1;
        if self.frame < 30 {
            return true;
        }
        let lang = if self.lang == Lang::En { "en" } else { "fr" };
        rl.take_screenshot(thread, &format!("shot_{}_{lang}.png", STEPS[self.step].0));
        self.frame = 0;
        self.step += 1;
        if self.step == STEPS.len() {
            if self.lang == Lang::Fr {
                return false;
            }
            self.lang = Lang::Fr;
            self.step = 0;
        }
        true
    }
}
