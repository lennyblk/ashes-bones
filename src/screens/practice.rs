//! terrain d'entraînement : on choisit une unité, on place des alliés et des ennemis
//! autour d'un duel, et on joue le mini-jeu avec exactement les conditions que le vrai
//! jeu donnerait (calculées par duel::compute sur de fausses unités). Solo seulement.

use raylib::prelude::*;

use crate::duel::{self, DuelConditions, DuelSide};
use crate::game::{Game, new_unit};
use crate::game_mode::GameMode;
use crate::input;
use crate::minigame::{MiniGame, MiniGameKind, TimingResult};
use crate::settings::{self, Bind};
use crate::ui::{self, HudRects};
use crate::unit::{Faction, Unit, UnitClass};
use crate::SCREEN_WIDTH;

pub const BOARD_COLS: usize = 7;
pub const BOARD_ROWS: usize = 5;
pub const CELL: f32 = 46.0;
/// case de l'attaquant (de loin pour un tireur qui vise), et de la cible
const ATTACKER: (i32, i32) = (2, 2);
const RANGED_ATTACKER: (i32, i32) = (1, 2);
const TARGET: (i32, i32) = (3, 2);
/// PV d'une unité « aux abois » (sous le seuil de 30%)
const LOW_HP_PCT: i32 = 20;

#[derive(Clone, Copy, PartialEq)]
pub enum Cell {
    Empty,
    Ally,
    Enemy,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Role {
    Attack,
    Defend,
}

pub enum Phase {
    Setup,
    /// `armed` passe à true à la frame suivante : le clic sur Play ne compte pas
    Playing { minigame: MiniGame, armed: bool },
    Result { result: TimingResult },
}

/// situations toutes prêtes, ouvertes depuis les cartes du guide
#[derive(Clone, Copy, PartialEq)]
pub enum Preset {
    MiniGame(MiniGameKind),
    Flank,
    Pincer,
    Support,
    Guarded,
    AimedShot,
    CaughtInMelee,
}

pub struct Practice {
    pub class: UnitClass,
    pub role: Role,
    /// [rangée][colonne]
    pub board: [[Cell; BOARD_COLS]; BOARD_ROWS],
    pub at_range: bool,
    pub me_low: bool,
    pub them_low: bool,
    pub phase: Phase,
    pub tries: u32,
    pub perfects: u32,
    pub streak: u32,
    pub best_streak: u32,
}

impl Practice {
    pub fn new() -> Practice {
        Practice {
            class: UnitClass::Soldier,
            role: Role::Attack,
            board: [[Cell::Empty; BOARD_COLS]; BOARD_ROWS],
            at_range: false,
            me_low: false,
            them_low: false,
            phase: Phase::Setup,
            tries: 0,
            perfects: 0,
            streak: 0,
            best_streak: 0,
        }
    }

    /// remet le terrain dans une situation du guide
    pub fn apply(&mut self, preset: Preset) {
        let keep_class = self.class;
        *self = Practice {
            tries: self.tries,
            perfects: self.perfects,
            best_streak: self.best_streak,
            ..Practice::new()
        };
        self.class = keep_class;
        let (tx, ty) = (TARGET.0 as usize, TARGET.1 as usize);
        match preset {
            Preset::MiniGame(kind) => self.class = kind.classes()[0],
            Preset::Flank => {
                self.board[ty - 1][tx] = Cell::Ally;
                self.board[ty + 1][tx] = Cell::Ally;
            }
            Preset::Pincer => self.board[ty][tx + 1] = Cell::Ally,
            Preset::Support => {
                self.board[ty - 1][ATTACKER.0 as usize] = Cell::Ally;
                self.board[ty + 1][ATTACKER.0 as usize] = Cell::Ally;
            }
            Preset::Guarded => self.board[ty - 1][tx] = Cell::Enemy,
            Preset::AimedShot => {
                self.class = UnitClass::Longbowman;
                self.at_range = true;
            }
            Preset::CaughtInMelee => {
                self.class = UnitClass::Longbowman;
                self.role = Role::Defend;
            }
        }
        self.fix_range();
    }

    pub fn kind(&self) -> MiniGameKind {
        MiniGameKind::for_class(self.class)
    }

    fn class_is_melee(&self) -> bool {
        new_unit(self.class).attack_range < 2
    }

    /// tirer de loin n'a de sens que pour un tireur qui attaque
    pub fn range_allowed(&self) -> bool {
        self.role == Role::Attack && !self.class_is_melee()
    }

    fn fix_range(&mut self) {
        if !self.range_allowed() {
            self.at_range = false;
        }
        // la case où se tient l'attaquant ne peut pas porter de jeton
        let (x, y) = self.attacker_tile();
        self.board[y as usize][x as usize] = Cell::Empty;
    }

    pub fn attacker_tile(&self) -> (i32, i32) {
        if self.at_range {
            RANGED_ATTACKER
        } else {
            ATTACKER
        }
    }

    pub fn target_tile(&self) -> (i32, i32) {
        TARGET
    }

    /// classes du duel (attaquant, défenseur). L'adversaire d'entraînement est un
    /// combattant au corps à corps du camp d'en face
    pub fn duel_classes(&self) -> (UnitClass, UnitClass) {
        let dummy = match self.class.faction() {
            Faction::Human => UnitClass::Skeleton,
            Faction::Undead => UnitClass::Soldier,
        };
        match self.role {
            Role::Attack => (self.class, dummy),
            Role::Defend => (dummy, self.class),
        }
    }

    /// fausses unités sur le plateau : attaquant (0), cible (1), puis les jetons
    fn units(&self) -> Vec<Unit> {
        let (attacker_class, defender_class) = self.duel_classes();
        let my_faction = self.class.faction();
        let place = |mut unit: Unit, (x, y): (i32, i32), low: bool| {
            unit.grid_x = x;
            unit.grid_y = y;
            if low {
                unit.hp_points = unit.hp_max_points * LOW_HP_PCT / 100;
            }
            unit
        };
        let (attacker_low, defender_low) = match self.role {
            Role::Attack => (self.me_low, self.them_low),
            Role::Defend => (self.them_low, self.me_low),
        };
        let mut units = vec![
            place(new_unit(attacker_class), self.attacker_tile(), attacker_low),
            place(new_unit(defender_class), TARGET, defender_low),
        ];
        for (y, row) in self.board.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let class = match (cell, my_faction) {
                    (Cell::Empty, _) => continue,
                    (Cell::Ally, Faction::Human)
                    | (Cell::Enemy, Faction::Undead) => UnitClass::Soldier,
                    _ => UnitClass::Skeleton,
                };
                units.push(place(new_unit(class), (x as i32, y as i32), false));
            }
        }
        units
    }

    pub fn conditions(&self) -> DuelConditions {
        duel::compute(&self.units(), 0, self.attacker_tile(), 1)
    }

    /// la barre que joue le joueur
    pub fn my_side(&self) -> DuelSide {
        *self.conditions().side(self.role == Role::Attack)
    }

    fn start(&mut self, rl: &RaylibHandle) {
        self.phase = Phase::Playing {
            minigame: MiniGame::new(self.kind(), &self.my_side(), rl),
            armed: false,
        };
    }

    fn finish(&mut self, result: TimingResult) {
        self.tries += 1;
        if matches!(result, TimingResult::Perfect) {
            self.perfects += 1;
            self.streak += 1;
            self.best_streak = self.best_streak.max(self.streak);
        } else {
            self.streak = 0;
        }
        self.phase = Phase::Result { result };
    }

    /// Échap : arrête l'essai en cours. Renvoie false s'il n'y en avait pas
    pub fn stop(&mut self) -> bool {
        if matches!(self.phase, Phase::Setup) {
            return false;
        }
        self.phase = Phase::Setup;
        true
    }
}

// mise en page (partagée entre update pour les clics et render pour le dessin) -----

pub const LEFT: Rectangle = Rectangle {
    x: 40.0,
    y: 78.0,
    width: 560.0,
    height: 600.0,
};
pub const RIGHT: Rectangle = Rectangle {
    x: 620.0,
    y: 78.0,
    width: 540.0,
    height: 600.0,
};

/// carte de la classe i (0..5 humains, 6..11 morts-vivants)
pub fn class_rect(i: usize) -> Rectangle {
    let size = 82.0;
    let gap = 8.0;
    let row = (i / 6) as f32;
    let col = (i % 6) as f32;
    Rectangle::new(
        LEFT.x + 22.0 + col * (size + gap),
        LEFT.y + 52.0 + row * (size + gap),
        size,
        size,
    )
}

pub fn all_classes() -> impl Iterator<Item = UnitClass> {
    UnitClass::HUMANS.into_iter().chain(UnitClass::UNDEAD)
}

pub fn role_rect() -> Rectangle {
    Rectangle::new(LEFT.x + 22.0, LEFT.y + 272.0, 320.0, 40.0)
}

pub fn board_origin() -> Vector2 {
    Vector2::new(LEFT.x + 22.0, LEFT.y + 360.0)
}

pub fn cell_rect(x: usize, y: usize) -> Rectangle {
    let origin = board_origin();
    Rectangle::new(
        origin.x + x as f32 * CELL,
        origin.y + y as f32 * CELL,
        CELL,
        CELL,
    )
}

/// cases à cocher à droite du plateau : de loin, mes PV bas, ses PV bas, puis Vider
pub fn option_rect(i: usize) -> Rectangle {
    let origin = board_origin();
    Rectangle::new(
        origin.x + BOARD_COLS as f32 * CELL + 20.0,
        origin.y + i as f32 * 50.0,
        190.0,
        40.0,
    )
}

pub fn play_rect() -> Rectangle {
    Rectangle::new(
        RIGHT.x + RIGHT.width / 2.0 - 120.0,
        RIGHT.y + RIGHT.height - 92.0,
        240.0,
        56.0,
    )
}

/// boutons de l'écran de résultat : rejouer / revenir au réglage
pub fn again_rect() -> Rectangle {
    Rectangle::new(SCREEN_WIDTH as f32 / 2.0 - 220.0, 330.0, 200.0, 52.0)
}

pub fn setup_rect() -> Rectangle {
    Rectangle::new(SCREEN_WIDTH as f32 / 2.0 + 20.0, 330.0, 200.0, 52.0)
}

pub fn update(
    game: &mut Game,
    rl: &RaylibHandle,
    hud: &HudRects,
    mouse: Vector2,
    click_consumed: bool,
    delta_time: f32,
) {
    if game.game_mode != GameMode::PracticeScreen {
        return;
    }
    let clicked = input::mouse_is_clicked(rl) && !click_consumed;
    let practice = &mut game.practice;

    match &mut practice.phase {
        Phase::Playing { minigame, armed } => {
            if !*armed {
                *armed = true;
                return;
            }
            minigame.update(rl, delta_time);
            if let Some(result) = minigame.result {
                practice.finish(result);
            }
            return;
        }
        Phase::Result { .. } => {
            let again = input::is_button_clicked(mouse, clicked, again_rect())
                || settings::pressed(rl, Bind::Action)
                || rl.is_key_pressed(KeyboardKey::KEY_ENTER);
            if again {
                practice.start(rl);
            } else if input::is_button_clicked(mouse, clicked, setup_rect()) {
                practice.phase = Phase::Setup;
            }
            return;
        }
        Phase::Setup => {}
    }

    if input::is_button_clicked(mouse, clicked, hud.btn_guide_back) {
        game.game_mode = GameMode::GuideScreen;
        return;
    }
    if !clicked {
        if settings::pressed(rl, Bind::Action) {
            practice.start(rl);
        }
        return;
    }

    for (i, class) in all_classes().enumerate() {
        if class_rect(i).check_collision_point_rec(mouse) {
            practice.class = class;
            practice.fix_range();
        }
    }
    for (i, cell) in ui::segment_rects(role_rect(), 2).iter().enumerate() {
        if cell.check_collision_point_rec(mouse) {
            practice.role = if i == 0 { Role::Attack } else { Role::Defend };
            practice.fix_range();
        }
    }
    for y in 0..BOARD_ROWS {
        for x in 0..BOARD_COLS {
            let tile = (x as i32, y as i32);
            if tile == practice.attacker_tile() || tile == TARGET {
                continue;
            }
            if cell_rect(x, y).check_collision_point_rec(mouse) {
                let cell = &mut practice.board[y][x];
                *cell = match cell {
                    Cell::Empty => Cell::Ally,
                    Cell::Ally => Cell::Enemy,
                    Cell::Enemy => Cell::Empty,
                };
            }
        }
    }
    if practice.range_allowed() && option_rect(0).check_collision_point_rec(mouse) {
        practice.at_range = !practice.at_range;
        practice.fix_range();
    }
    if option_rect(1).check_collision_point_rec(mouse) {
        practice.me_low = !practice.me_low;
    }
    if option_rect(2).check_collision_point_rec(mouse) {
        practice.them_low = !practice.them_low;
    }
    if option_rect(3).check_collision_point_rec(mouse) {
        practice.board = [[Cell::Empty; BOARD_COLS]; BOARD_ROWS];
        practice.me_low = false;
        practice.them_low = false;
    }
    if play_rect().check_collision_point_rec(mouse) {
        practice.start(rl);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::duel::FactorKind;

    fn kinds(preset: Preset) -> Vec<FactorKind> {
        let mut practice = Practice::new();
        practice.apply(preset);
        practice.conditions().factors.iter().map(|f| f.kind).collect()
    }

    #[test]
    fn presets_give_their_factor() {
        assert!(kinds(Preset::Flank).contains(&FactorKind::Flank(2)));
        assert!(kinds(Preset::Pincer).contains(&FactorKind::Pincer));
        assert!(kinds(Preset::Support).contains(&FactorKind::Support(2)));
        assert!(kinds(Preset::Guarded).contains(&FactorKind::Guarded(1)));
        assert!(kinds(Preset::AimedShot).contains(&FactorKind::AimedShot));
        assert!(kinds(Preset::CaughtInMelee).contains(&FactorKind::CaughtInMelee));
        assert!(kinds(Preset::MiniGame(MiniGameKind::Timing)).is_empty());
    }

    #[test]
    fn defending_plays_the_defender_bar() {
        let mut practice = Practice::new();
        practice.apply(Preset::CaughtInMelee);
        let conditions = practice.conditions();
        assert_eq!(practice.my_side().window, conditions.defender.window);
        assert!(practice.my_side().window < DuelSide::base().window);
    }

    #[test]
    fn melee_units_cannot_shoot_from_range() {
        let mut practice = Practice::new();
        practice.apply(Preset::AimedShot);
        assert!(practice.at_range);
        practice.class = UnitClass::Soldier;
        practice.fix_range();
        assert!(!practice.at_range);
    }
}
