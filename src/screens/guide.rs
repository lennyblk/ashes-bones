//! guide en 3 pages courtes et visuelles (bases, mini-jeux, placement). Chaque carte de
//! mini-jeu ou de placement s'essaie sur le terrain d'entraînement (solo seulement)

use raylib::prelude::*;

use crate::SCREEN_WIDTH;
use crate::game::Game;
use crate::game_mode::GameMode;
use crate::input;
use crate::minigame::MiniGameKind;
use crate::screens::practice::Preset;
use crate::settings::tr;
use crate::ui::HudRects;

#[derive(Clone, Copy, PartialEq)]
pub enum GuidePage {
    Basics,
    MiniGames,
    Positioning,
}

impl GuidePage {
    pub const ALL: [GuidePage; 3] = [GuidePage::Basics, GuidePage::MiniGames, GuidePage::Positioning];

    pub fn title(self) -> &'static str {
        match self {
            GuidePage::Basics => tr("Basics", "Les bases"),
            GuidePage::MiniGames => tr("Mini-games", "Mini-jeux"),
            GuidePage::Positioning => tr("Positioning", "Placement"),
        }
    }
}

/// une carte de la page Placement : la situation, sa mini grille et ce qu'elle change
pub struct PositionCard {
    pub preset: Preset,
    /// A = attaquant, a = allié, E = cible, e = allié de la cible, . = vide
    pub rows: &'static [&'static str],
    pub title: &'static str,
    pub lines: [&'static str; 2],
}

pub fn position_cards() -> [PositionCard; 6] {
    [
        PositionCard {
            preset: Preset::Flank,
            rows: &[". a .", "A E .", ". a ."],
            title: tr("Flank", "Encerclement"),
            lines: [
                tr("Allies touching the target:", "Alliés collés à la cible :"),
                tr("bigger yellow zone for you.", "ta zone jaune grandit."),
            ],
        },
        PositionCard {
            preset: Preset::Pincer,
            rows: &["A E a"],
            title: tr("Pincer", "Tenaille"),
            lines: [
                tr("An ally right behind it:", "Un allié pile derrière :"),
                tr("no PERFECT parry for them.", "pas de parade PARFAITE."),
            ],
        },
        PositionCard {
            preset: Preset::Support,
            rows: &["a . .", "A E .", "a . ."],
            title: tr("Support", "Soutien"),
            lines: [
                tr("Allies touching YOU:", "Alliés collés à TOI :"),
                tr("+1 life each, a miss is saved.", "+1 vie chacun, un raté sauvé."),
            ],
        },
        PositionCard {
            preset: Preset::Guarded,
            rows: &[". e .", "A E ."],
            title: tr("Guarded", "Protégé"),
            lines: [
                tr("Enemies touching the target:", "Ennemis collés à la cible :"),
                tr("smaller zone, they get lives.", "zone réduite, il a des vies."),
            ],
        },
        PositionCard {
            preset: Preset::AimedShot,
            rows: &["A . E"],
            title: tr("Aimed shot", "Tir posé"),
            lines: [
                tr("Shooters hit from afar:", "Les tireurs frappent de loin :"),
                tr("slower cursor. Close = harder.", "curseur lent. Au contact = dur."),
            ],
        },
        PositionCard {
            preset: Preset::CaughtInMelee,
            rows: &["A E"],
            title: tr("Shooter in melee", "Tireur au contact"),
            lines: [
                tr("Shooter attacked up close:", "Tireur attaqué de près :"),
                tr("its zone shrinks and speeds up.", "sa zone rétrécit et accélère."),
            ],
        },
    ]
}

// mise en page (partagée entre update pour les clics et render pour le dessin) -----

pub const CONTENT_TOP: f32 = 150.0;

/// onglets : les 3 pages, puis l'entraînement
pub fn tab_rects() -> [Rectangle; 4] {
    let width = 230.0;
    let gap = 12.0;
    let total = width * 4.0 + gap * 3.0;
    let x0 = SCREEN_WIDTH as f32 / 2.0 - total / 2.0;
    std::array::from_fn(|i| Rectangle::new(x0 + i as f32 * (width + gap), 88.0, width, 46.0))
}

/// 4 étapes de la page Bases
pub fn step_rect(i: usize) -> Rectangle {
    let width = 262.0;
    let gap = 18.0;
    let x0 = SCREEN_WIDTH as f32 / 2.0 - (width * 4.0 + gap * 3.0) / 2.0;
    Rectangle::new(x0 + i as f32 * (width + gap), CONTENT_TOP, width, 250.0)
}

/// bas de la page Bases : résultats à gauche, touches à droite
pub fn basics_bottom_rects() -> (Rectangle, Rectangle) {
    let first = step_rect(0);
    let last = step_rect(3);
    let top = first.y + first.height + 16.0;
    let full = last.x + last.width - first.x;
    let half = (full - 18.0) / 2.0;
    (
        Rectangle::new(first.x, top, half, 270.0),
        Rectangle::new(first.x + half + 18.0, top, half, 270.0),
    )
}

/// cartes des mini-jeux : 4 en haut, 3 en bas
pub fn minigame_rect(i: usize) -> Rectangle {
    let width = 268.0;
    let height = 258.0;
    let gap = 16.0;
    let per_row = if i < 4 { 4 } else { 3 };
    let col = if i < 4 { i } else { i - 4 } as f32;
    let row = if i < 4 { 0.0 } else { 1.0 };
    let x0 = SCREEN_WIDTH as f32 / 2.0 - (width * per_row as f32 + gap * (per_row - 1) as f32) / 2.0;
    Rectangle::new(
        x0 + col * (width + gap),
        CONTENT_TOP + row * (height + gap),
        width,
        height,
    )
}

/// cartes du placement : 3 x 2
pub fn position_rect(i: usize) -> Rectangle {
    let width = 366.0;
    let height = 236.0;
    let gap = 16.0;
    let col = (i % 3) as f32;
    let row = (i / 3) as f32;
    let x0 = SCREEN_WIDTH as f32 / 2.0 - (width * 3.0 + gap * 2.0) / 2.0;
    Rectangle::new(
        x0 + col * (width + gap),
        CONTENT_TOP + row * (height + gap),
        width,
        height,
    )
}

/// bouton « Essayer » en bas d'une carte
pub fn try_rect(card: Rectangle) -> Rectangle {
    Rectangle::new(card.x + card.width / 2.0 - 80.0, card.y + card.height - 50.0, 160.0, 40.0)
}

pub fn open(game: &mut Game, from: GameMode) {
    game.guide_return_to = from;
    game.game_mode = GameMode::GuideScreen;
}

fn open_practice(game: &mut Game, preset: Option<Preset>) {
    // l'entraînement prend la souris et le clavier : impossible pendant une partie en ligne
    if game.online {
        return;
    }
    if let Some(preset) = preset {
        game.practice.apply(preset);
    }
    game.game_mode = GameMode::PracticeScreen;
}

pub fn update(
    game: &mut Game,
    rl: &RaylibHandle,
    hud: &HudRects,
    mouse: Vector2,
    click_consumed: &mut bool,
) {
    if game.game_mode != GameMode::GuideScreen {
        return;
    }
    let clicked = input::mouse_is_clicked(rl) && !*click_consumed;
    if input::is_button_clicked(mouse, clicked, hud.btn_guide_back) {
        game.game_mode = game.guide_return_to;
        *click_consumed = true;
        return;
    }
    if !clicked {
        return;
    }
    // le clic qui ouvre l'entraînement ne doit pas y compter aussi
    *click_consumed = true;
    let tabs = tab_rects();
    for (page, tab) in GuidePage::ALL.iter().zip(tabs) {
        if tab.check_collision_point_rec(mouse) {
            game.guide_page = *page;
        }
    }
    if tabs[3].check_collision_point_rec(mouse) {
        open_practice(game, None);
        return;
    }
    match game.guide_page {
        GuidePage::Basics => {}
        GuidePage::MiniGames => {
            for (i, kind) in MiniGameKind::ALL.iter().enumerate() {
                if minigame_rect(i).check_collision_point_rec(mouse) {
                    open_practice(game, Some(Preset::MiniGame(*kind)));
                }
            }
        }
        GuidePage::Positioning => {
            for (i, card) in position_cards().iter().enumerate() {
                if position_rect(i).check_collision_point_rec(mouse) {
                    open_practice(game, Some(card.preset));
                }
            }
        }
    }
}
