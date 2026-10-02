mod target;
mod timing;

use crate::SCREEN_WIDTH;
use crate::animation::Animation;
use crate::assets::Assets;
use crate::duel::{DuelConditions, DuelSide};
use crate::unit::{Faction, PendingAction, Unit, UnitClass, UnitState};
use raylib::prelude::*;

use target::TargetShot;
use timing::TimingBar;

// temps max pour réussir un mini-jeu, après c'est BAD
pub const TIME_LIMIT: f32 = 4.0;
const SAVED_FLASH: f32 = 0.6;

#[derive(Clone, Copy)]
pub enum TimingResult {
    Bad,
    Good,
    Perfect,
}

impl TimingResult {
    pub fn to_multiplier(&self, is_attacking: bool) -> f32 {
        let base = match self {
            TimingResult::Bad => 0.5,
            TimingResult::Good => 1.0,
            TimingResult::Perfect => 1.5,
        };

        if is_attacking { base } else { 2.0 - base }
    }
}

/// ce qu'un mini-jeu renvoie quand le joueur tente quelque chose.
/// Un Miss n'est pas forcément BAD : MiniGame le pardonne s'il reste des vies
pub enum Attempt {
    Miss,
    Hit(TimingResult),
}

/// quel mini-jeu joue une classe (en attaque comme en défense)
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MiniGameKind {
    Timing,
    Target,
}

impl MiniGameKind {
    pub fn for_class(class: UnitClass) -> MiniGameKind {
        match class {
            UnitClass::Soldier
            | UnitClass::Skeleton
            | UnitClass::Cavalry
            | UnitClass::Ghoul
            | UnitClass::Assassin
            | UnitClass::Wraith
            | UnitClass::Mage
            | UnitClass::Banshee
            | UnitClass::BloodKnight
            | UnitClass::Priest
            | UnitClass::Necromancer => MiniGameKind::Timing,
            UnitClass::Longbowman => MiniGameKind::Target,
        }
    }
}

/// la mécanique propre à chaque mini-jeu. Pour en ajouter un : un fichier avec
/// new(rl, side) / update(rl, dt) -> Option<Attempt> / draw(d) / area(), puis une variante ici
enum Mechanic {
    Timing(TimingBar),
    Target(TargetShot),
}

/// partie commune à tous les mini-jeux : chrono, vies, résultat final
pub struct MiniGame {
    mechanic: Mechanic,
    pub time_remaining: f32,
    pub lives: u8,
    // > 0 pendant qu'on affiche "SAVED" après un raté pardonné
    pub saved_timer: f32,
    pub result: Option<TimingResult>,
}

impl MiniGame {
    pub fn new(kind: MiniGameKind, side: &DuelSide, rl: &RaylibHandle) -> MiniGame {
        let mechanic = match kind {
            MiniGameKind::Timing => Mechanic::Timing(TimingBar::new(rl, side)),
            MiniGameKind::Target => Mechanic::Target(TargetShot::new(rl, side)),
        };
        MiniGame {
            mechanic,
            time_remaining: TIME_LIMIT,
            lives: side.lives,
            saved_timer: 0.0,
            result: None,
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle, delta_time: f32) {
        self.saved_timer = (self.saved_timer - delta_time).max(0.0);
        if self.result.is_some() {
            return;
        }

        let attempt = match &mut self.mechanic {
            Mechanic::Timing(bar) => bar.update(rl, delta_time),
            Mechanic::Target(target) => target.update(rl, delta_time),
        };
        match attempt {
            Some(Attempt::Hit(result)) => self.result = Some(result),
            Some(Attempt::Miss) if self.lives > 0 => {
                self.lives -= 1;
                self.saved_timer = SAVED_FLASH;
            }
            Some(Attempt::Miss) => self.result = Some(TimingResult::Bad),
            None => {}
        }

        self.time_remaining -= delta_time;
        if self.result.is_none() && self.time_remaining <= 0.0 {
            self.result = Some(TimingResult::Bad);
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle, assets: &Assets) {
        let area = match &self.mechanic {
            Mechanic::Timing(bar) => {
                bar.draw(d);
                bar.area()
            }
            Mechanic::Target(target) => {
                target.draw(d);
                target.area()
            }
        };
        self.draw_lives(d, assets, area);
    }

    /// vies en carrés rouges à gauche de l'aire du mini-jeu, "SAVED!" au-dessus
    fn draw_lives(&self, d: &mut RaylibDrawHandle, assets: &Assets, area: Rectangle) {
        let middle_y = area.y + area.height / 2.0;
        for i in 0..self.lives {
            d.draw_rectangle(
                (area.x - 22.0 - i as f32 * 18.0) as i32,
                (middle_y - 7.0) as i32,
                14,
                14,
                Color::RED,
            );
        }
        if self.saved_timer > 0.0 {
            let text = "SAVED!";
            let size = assets.info_font.measure_text(text, 28.0, 1.0);
            d.draw_text_ex(
                &assets.info_font,
                text,
                Vector2::new(SCREEN_WIDTH as f32 / 2.0 - size.x / 2.0, area.y - 40.0),
                28.0,
                1.0,
                Color::ORANGE,
            );
        }
    }
}

pub fn handle_minigame(
    attacker: &mut Unit,
    defender: &mut Unit,
    player_faction: Faction,
    minigame: &mut Option<MiniGame>,
    conditions: &DuelConditions,
    damage_multiplier: &mut f32,
    rl: &RaylibHandle,
    delta_time: f32,
    attacker_attack_animation: &mut Animation,
) -> Option<TimingResult> {
    if attacker.state != UnitState::MiniGame {
        return None;
    }

    let player_is_attacker = attacker.faction == player_faction;

    // le joueur joue le mini-jeu de SA classe, qu'il attaque ou qu'il défende
    if minigame.is_none() {
        let player_class = if player_is_attacker {
            attacker.class
        } else {
            defender.class
        };
        *minigame = Some(MiniGame::new(
            MiniGameKind::for_class(player_class),
            conditions.side(player_is_attacker),
            rl,
        ));
    }

    let game = minigame.as_mut().unwrap();
    game.update(rl, delta_time);

    if let Some(result) = game.result {
        *damage_multiplier = result.to_multiplier(player_is_attacker);
        attacker.state = match attacker.pending_action {
            Some(PendingAction::Heal(_)) => UnitState::Healing,
            _ => UnitState::Attacking,
        };
        attacker_attack_animation.current = 0;
        attacker_attack_animation.finished = false;
        *minigame = None;
        return Some(result);
    }
    None
}
