use raylib::prelude::*;

use crate::SCREEN_WIDTH;
use crate::duel::{
    AIMED_TEMPO, BASE_PRECISION, BASE_WINDOW, DESPERATE_HP_PCT, FLANK_WINDOW, GUARD_WINDOW,
    MAX_FLANKERS, MAX_LIVES, MAX_WINDOW, MELEE_PANIC_TEMPO, MIN_WINDOW, PRECISION_BONUS,
    SHOOTER_IN_MELEE_WINDOW,
};
use crate::game::Game;
use crate::game_mode::GameMode;
use crate::input;
use crate::minigame::{RUNES_TIME, SEQUENCE_TIME, TIME_LIMIT};
use crate::ui::HudRects;

// mise en page du guide (partagée entre update pour le scroll et render pour le dessin)
pub const PANEL_PAD: f32 = 32.0;
// bande du haut réservée au titre + persos idle
pub const HEADER_HEIGHT: f32 = 90.0;
const SIDE_MARGIN: f32 = 40.0;
pub const SCROLLBAR_WIDTH: f32 = 12.0;
pub const DIAGRAM_CELL: f32 = 34.0;
pub const HEADING_SIZE: f32 = 30.0;
pub const TEXT_SIZE: f32 = 20.0;
pub const CAPTION_SIZE: f32 = 18.0;
const WHEEL_STEP: f32 = 50.0;

pub const GOOD: Color = Color::new(120, 230, 120, 255);
pub const BAD: Color = Color::new(255, 110, 110, 255);
pub const MUTED: Color = Color::new(180, 180, 180, 255);

/// un morceau du guide, dessiné de haut en bas
pub enum Block {
    Heading(String),
    Text(String),
    Gap,
    /// mini grille : A = attaquant, a = allié, E = cible, e = ennemi, . = vide,
    /// 2 / 1 / 0 / n = case jaune avec un badge +2 / +1 / 0 / -1
    Diagram {
        rows: Vec<&'static str>,
        caption: Vec<String>,
    },
    /// barre de timing d'exemple (window / precision en 0..1, lives = carrés rouges)
    Bar {
        window: f32,
        precision: f32,
        cursor: f32,
        lives: u8,
        caption: String,
    },
    /// faux panneau de preview, lignes colorées
    Panel(Vec<(String, Color)>),
    /// petite image d'un mini-jeu, dessinée à droite des lignes qui suivent.
    /// Ne prend pas de place dans la colonne de texte
    Illustration(Illustration),
}

/// mini-jeux illustrés dans le guide (vue figée, mêmes couleurs que le vrai jeu)
pub enum Illustration {
    Target,
    Charge,
    Sequence,
    Runes,
    Heartbeat,
    Channel,
}

pub const ILLUSTRATION_WIDTH: f32 = 300.0;
pub const ILLUSTRATION_HEIGHT: f32 = 100.0;

impl Block {
    pub fn height(&self) -> f32 {
        match self {
            Block::Heading(_) => 48.0,
            Block::Text(_) => 26.0,
            Block::Gap => 14.0,
            Block::Diagram { rows, caption } => {
                let grid = rows.len() as f32 * DIAGRAM_CELL;
                let text = caption.len() as f32 * 24.0;
                grid.max(text) + 18.0
            }
            Block::Bar { .. } => 54.0,
            Block::Panel(lines) => lines.len() as f32 * 22.0 + 30.0,
            Block::Illustration(_) => 0.0,
        }
    }

    /// hauteur réellement dessinée (une illustration déborde sur les lignes suivantes)
    pub fn draw_extent(&self) -> f32 {
        match self {
            Block::Illustration(_) => ILLUSTRATION_HEIGHT,
            _ => self.height(),
        }
    }
}

fn pct(value: f32) -> i32 {
    (value * 100.0).round() as i32
}

fn text(s: &str) -> Block {
    Block::Text(String::from(s))
}

fn heading(s: &str) -> Block {
    Block::Heading(String::from(s))
}

fn diagram(rows: Vec<&'static str>, caption: Vec<String>) -> Block {
    Block::Diagram { rows, caption }
}

/// tout le texte du guide. Les chiffres viennent de duel.rs pour rester à jour
pub fn content() -> Vec<Block> {
    vec![
        heading("Basics"),
        text("Left click one of your units to select it."),
        text("Left click a blue tile to move there."),
        text("Left click a red enemy to attack it."),
        text("Priests and Necromancers click a green ally to heal it."),
        text("Several tiles can reach the target? Pick one (yellow)."),
        text("B cancels an action, Escape opens the pause menu."),
        text("x1/x2 button (or F) speeds up the map, combats stay normal."),
        text("Win by wiping out every enemy unit."),
        Block::Gap,
        heading("Combat: the minigame"),
        text("Every attack opens a duel: a short minigame sets the damage."),
        text("Each class has its own minigame. Most use the timing bar:"),
        text("click while the white cursor is inside the yellow zone."),
        Block::Bar {
            window: BASE_WINDOW,
            precision: BASE_PRECISION,
            cursor: 0.3,
            lives: 0,
            caption: String::from("yellow = GOOD, green = PERFECT"),
        },
        text("You attack:  BAD x0.5   GOOD x1   PERFECT x1.5 damage dealt"),
        text("You defend:  the enemy attacks, you play the bar to parry"),
        text("             BAD x1.5   GOOD x1   PERFECT x0.5 damage taken"),
        Block::Text(format!(
            "You have {} seconds (Assassin/Wraith: {}, Mage/Banshee: {}).",
            TIME_LIMIT as i32, SEQUENCE_TIME as i32, RUNES_TIME as i32
        )),
        text("Out of time = BAD."),
        text("You play the minigame of the unit YOU control, even on defense."),
        Block::Gap,
        Block::Illustration(Illustration::Target),
        text("Longbowman: shoot the moving target with the crosshair."),
        text("  yellow ring = GOOD, green center = PERFECT, outside = miss."),
        text("  Window = target size, precision = center size,"),
        text("  tempo = how fast the target moves."),
        Block::Gap,
        Block::Illustration(Illustration::Charge),
        text("Cavalry / Ghoul: charge! A white ring closes in on the target."),
        text("  Click when it crosses the yellow band (green = PERFECT)."),
        text("  Too late: the ring passes, a miss (a life starts a new charge)."),
        text("  Window = band size, precision = green size, tempo = ring speed."),
        Block::Gap,
        Block::Illustration(Illustration::Sequence),
        text("Assassin / Wraith: type the 4 arrows in order, fast!"),
        text("  Arrow keys or WASD (ZQSD on AZERTY). Finish in the green part"),
        text("  of the time bar = PERFECT, in the yellow part = GOOD."),
        text("  Wrong key = miss. Too slow = miss (a life gives a new sequence)."),
        text("  Window = time allowed, precision = green part, tempo = speed."),
        Block::Gap,
        Block::Illustration(Illustration::Runes),
        text("Mage / Banshee: watch 4 runes light up, then click them back"),
        text("  in the same order. Same time bar: green = PERFECT, yellow = GOOD."),
        text("  Wrong rune = miss. Too slow = miss (a life shows a new sequence)."),
        text("  The timer is paused while the runes are shown."),
        text("  Window = time to repeat, precision = green part, tempo = speed."),
        Block::Gap,
        Block::Illustration(Illustration::Heartbeat),
        text("Blood Knight: blood pulse! The heart beats twice to set the rhythm,"),
        text("  then click on each of the 3 blood drops as it crosses the line."),
        text("  Every drop must land in the yellow band. Average offset in the"),
        text("  green band = PERFECT. A missed drop = miss (a life skips it)."),
        text("  Window = yellow band, precision = green band, tempo = heart speed."),
        Block::Gap,
        Block::Illustration(Illustration::Channel),
        text("Priest / Necromancer: channel! Keep your cursor inside the drifting"),
        text("  aura for 3 seconds, no click. Green core = full focus,"),
        text("  yellow = 60%. Final focus: 50%+ = GOOD, 85%+ = PERFECT."),
        text("  Not enough focus = miss (a life restarts the channel)."),
        text("  Window = aura size, precision = core size, tempo = drift speed."),
        Block::Gap,
        heading("Duel conditions"),
        text("Where your units stand shapes the minigame BEFORE it starts."),
        text("The grid sets the conditions, the minigame sets the result."),
        text("Each side of a duel has 4 settings:"),
        Block::Text(format!(
            "  Window     size of the yellow zone (base {}%)",
            pct(BASE_WINDOW)
        )),
        Block::Text(format!(
            "  Precision  size of the green zone (base {}%)",
            pct(BASE_PRECISION)
        )),
        text("  Tempo      speed of the cursor (lower = calmer)"),
        text("  Lives      missed clicks that get forgiven"),
        Block::Bar {
            window: (BASE_WINDOW + 2.0 * FLANK_WINDOW).min(MAX_WINDOW),
            precision: BASE_PRECISION + PRECISION_BONUS,
            cursor: 0.75,
            lives: 0,
            caption: String::from("same bar after a good setup"),
        },
        Block::Gap,
        heading("Positioning"),
        diagram(
            vec!["A a E e"],
            vec![String::from("A you   a your ally   E target   e its ally")],
        ),
        diagram(
            vec![". a .", "A E a", ". . ."],
            vec![
                String::from("FLANK: your allies touching the target"),
                format!(
                    "you: +{}% window per ally (max {})",
                    pct(FLANK_WINDOW),
                    MAX_FLANKERS
                ),
                format!("target: -{}% window per ally", pct(FLANK_WINDOW)),
            ],
        ),
        diagram(
            vec!["A E a"],
            vec![
                String::from("PINCER: an ally exactly opposite you"),
                format!("you: +{}% precision", pct(PRECISION_BONUS)),
                String::from("target: no PERFECT parry possible"),
            ],
        ),
        diagram(
            vec!["a . .", "A E ."],
            vec![
                String::from("SUPPORT: your allies touching YOUR tile"),
                format!("you: +1 life per ally (max {})", MAX_LIVES),
            ],
        ),
        diagram(
            vec!["A E .", ". e ."],
            vec![
                String::from("GUARDED: enemies touching the target"),
                format!(
                    "you: -{}% window per guard (max {})",
                    pct(GUARD_WINDOW),
                    MAX_LIVES
                ),
                String::from("target: +1 life per guard"),
            ],
        ),
        diagram(
            vec!["A . E"],
            vec![
                String::from("AIMED SHOT: a shooter hitting from range"),
                format!("you: tempo -{}% (calmer cursor)", pct(1.0 - AIMED_TEMPO)),
                format!(
                    "POINT BLANK: shooting when adjacent, -{}% window",
                    pct(SHOOTER_IN_MELEE_WINDOW)
                ),
            ],
        ),
        diagram(
            vec!["A E"],
            vec![
                String::from("CAUGHT IN MELEE: E is a shooter, you are adjacent"),
                format!(
                    "target: -{}% window, tempo +{}%",
                    pct(SHOOTER_IN_MELEE_WINDOW),
                    pct(MELEE_PANIC_TEMPO - 1.0)
                ),
            ],
        ),
        Block::Text(format!(
            "DESPERATE: under {}% HP, +{}% precision (works for both sides).",
            DESPERATE_HP_PCT,
            pct(PRECISION_BONUS)
        )),
        Block::Text(format!(
            "Limits: window stays between {}% and {}%, max {} lives.",
            pct(MIN_WINDOW),
            pct(MAX_WINDOW),
            MAX_LIVES
        )),
        text("Shooters = units with range 2 or more (Longbowman, Mage...)."),
        Block::Gap,
        heading("Lives"),
        Block::Bar {
            window: BASE_WINDOW,
            precision: BASE_PRECISION,
            cursor: 0.85,
            lives: 2,
            caption: String::from("red squares = lives left"),
        },
        text("A miss with a life left is forgiven: SAVED!"),
        text("You don't restart: the same bar keeps going,"),
        text("click again when the cursor comes back in the zone."),
        Block::Text(format!(
            "The {} second timer does NOT reset. Too late = BAD.",
            TIME_LIMIT as i32
        )),
        text("No life left: your first miss is a BAD."),
        text("Lives come from allies standing next to you, any class."),
        text("Heals always use a neutral bar: no bonus, no lives."),
        Block::Gap,
        heading("Reading the duel preview"),
        text("Select a unit and hover an enemy: the duel shows up in advance."),
        Block::Panel(vec![
            (String::from("Soldier -> Wraith"), Color::WHITE),
            (
                String::from("DMG  bad 20  good 41  perfect 61"),
                Color::WHITE,
            ),
            (
                String::from("Wraith HP 55 -> 14 (good)   KO on PERFECT"),
                Color::ORANGE,
            ),
            (
                String::from("Expected ~42 dmg   odds: perfect 21%  good 65%  bad 14%"),
                Color::WHITE,
            ),
            (String::from("Placement: +25% damage vs no modifiers"), GOOD),
            (
                String::from("You    window 25%  precision 3%  tempo x1.0  lives 1"),
                Color::WHITE,
            ),
            (
                String::from("Them   window 12%  precision 3%  tempo x1.0  lives 1"),
                MUTED,
            ),
            (
                String::from("+ Flank x2: window +8% / their window -8%"),
                GOOD,
            ),
            (String::from("+ Support x1: +1 lives"), GOOD),
            (
                String::from("- Guarded x1: window -3% / they get +1 lives"),
                BAD,
            ),
        ]),
        text("DMG: damage for each minigame result."),
        text("HP: target HP after a GOOD, and if a KO is possible."),
        text("Expected / odds: estimated chances with YOUR bar (rough guess)."),
        text("Placement: how much more (or less) damage this setup gives you"),
        text("on average, compared to a duel with no modifiers."),
        text("You / Them: both bars and their 4 settings."),
        text("Green lines help you, red lines hurt you."),
        text("Border: green = the placement helps you, red = it hurts you."),
        text("(best tile): you must move first, a white frame shows where."),
        text("'Them' is for info: for now only you play a bar."),
        Block::Gap,
        heading("Choosing your attack tile"),
        diagram(
            vec![". 1 .", "0 E 2", ". n ."],
            vec![
                String::from("Each yellow tile shows its Placement bonus:"),
                String::from("+25 = 25% more damage on average from there."),
                String::from("Hover a tile to see the full preview."),
            ],
        ),
        text("Factors are weighed: a big window boost or extra lives count more"),
        text("than a small tempo change. Same number as the preview's Placement."),
        Block::Gap,
        heading("Tips"),
        text("Enemies play the same game: they flank, pincer and finish"),
        text("wounded units. Isolated units are their favorite targets."),
        text("Gang up: flank + pincer turns a hard duel into an easy one."),
        text("Keep your shooters away from enemy melee units."),
        text("Leave an ally next to fragile units: they get lives on defense."),
        text("Hit lone enemies first, guarded ones are harder to hit."),
        Block::Gap,
    ]
}

/// zone du texte qui scrolle : presque tout l'écran, entre le titre et le bouton Back
pub fn panel_rect(hud: &HudRects) -> Rectangle {
    Rectangle {
        x: SIDE_MARGIN,
        y: HEADER_HEIGHT,
        width: SCREEN_WIDTH as f32 - SIDE_MARGIN * 2.0,
        height: hud.btn_guide_back.y - 12.0 - HEADER_HEIGHT,
    }
}

pub fn scrollbar_track(hud: &HudRects) -> Rectangle {
    let panel = panel_rect(hud);
    Rectangle {
        x: panel.x + panel.width - SCROLLBAR_WIDTH - 6.0,
        y: panel.y + 6.0,
        width: SCROLLBAR_WIDTH,
        height: panel.height - 12.0,
    }
}

pub fn content_height() -> f32 {
    PANEL_PAD * 2.0 + content().iter().map(Block::height).sum::<f32>()
}

pub fn max_scroll(hud: &HudRects) -> f32 {
    (content_height() - panel_rect(hud).height).max(0.0)
}

/// poignée de la scrollbar, sa taille montre la part du guide visible
pub fn scrollbar_thumb(hud: &HudRects, scroll: f32) -> Rectangle {
    let track = scrollbar_track(hud);
    let visible = (panel_rect(hud).height / content_height()).min(1.0);
    let height = (track.height * visible).max(30.0);
    let progress = if max_scroll(hud) > 0.0 {
        scroll / max_scroll(hud)
    } else {
        0.0
    };
    Rectangle {
        x: track.x,
        y: track.y + (track.height - height) * progress,
        width: track.width,
        height,
    }
}

pub fn update(game: &mut Game, rl: &RaylibHandle, hud: &HudRects, mouse_position: Vector2) {
    if game.game_mode != GameMode::GuideScreen {
        return;
    }
    if input::is_button_clicked(
        mouse_position,
        input::mouse_is_clicked(rl),
        hud.btn_guide_back,
    ) {
        game.game_mode = game.guide_return_to;
        return;
    }

    // molette
    game.guide_scroll -= rl.get_mouse_wheel_move() * WHEEL_STEP;

    // drag de la scrollbar : clic sur la piste puis on suit la souris
    let track = scrollbar_track(hud);
    if input::mouse_is_clicked(rl) && track.check_collision_point_rec(mouse_position) {
        game.guide_dragging = true;
    }
    if !rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
        game.guide_dragging = false;
    }
    if game.guide_dragging {
        let thumb = scrollbar_thumb(hud, game.guide_scroll);
        let free_space = track.height - thumb.height;
        if free_space > 0.0 {
            let progress = (mouse_position.y - track.y - thumb.height / 2.0) / free_space;
            game.guide_scroll = progress * max_scroll(hud);
        }
    }

    game.guide_scroll = game.guide_scroll.clamp(0.0, max_scroll(hud));
}
