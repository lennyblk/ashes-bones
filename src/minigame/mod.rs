mod channel;
mod charge;
mod heartbeat;
mod runes;
mod sequence;
mod target;
mod timing;

use crate::SCREEN_WIDTH;
use crate::animation::Animation;
use crate::assets::Assets;
use crate::duel::{DuelConditions, DuelSide};
use crate::unit::{Faction, PendingAction, Unit, UnitClass, UnitState};
use raylib::prelude::*;

use channel::Channel;
use charge::ChargeRing;
use heartbeat::Heartbeat;
pub use runes::BASE_GOOD_TIME as RUNES_TIME;
use runes::RuneMemory;
pub use sequence::BASE_GOOD_TIME as SEQUENCE_TIME;
use sequence::KeySequence;
use target::TargetShot;
use timing::TimingBar;

// temps max pour réussir un mini-jeu, après c'est BAD. La séquence, les runes, le
// cœur et la canalisation gèrent leur propre temps et n'utilisent pas celui-ci
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

// part du temps accordé qui donne PERFECT, par point de precision/window.
// Base (3% / 20%) -> 2.5 x 0.15 = ~37% du temps
const PERFECT_SHARE_SCALE: f32 = 2.5;

/// pour les mini-jeux « finir avant la fin de la barre » (séquence, runes) :
/// temps max pour un PERFECT. 0 = PERFECT impossible (ex : défenseur pris à revers)
fn perfect_time(side: &DuelSide, good_time: f32) -> f32 {
    if side.precision <= 0.0 {
        return 0.0;
    }
    good_time * (PERFECT_SHARE_SCALE * side.precision / side.window).min(0.95)
}

/// quel mini-jeu joue une classe (en attaque comme en défense)
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MiniGameKind {
    Timing,
    Target,
    Charge,
    Sequence,
    Runes,
    Heartbeat,
    Channel,
}

impl MiniGameKind {
    pub fn for_class(class: UnitClass) -> MiniGameKind {
        match class {
            UnitClass::Soldier | UnitClass::Skeleton => MiniGameKind::Timing,
            UnitClass::Priest | UnitClass::Necromancer => MiniGameKind::Channel,
            UnitClass::Longbowman => MiniGameKind::Target,
            UnitClass::Cavalry | UnitClass::Ghoul => MiniGameKind::Charge,
            UnitClass::Assassin | UnitClass::Wraith => MiniGameKind::Sequence,
            UnitClass::Mage | UnitClass::Banshee => MiniGameKind::Runes,
            UnitClass::BloodKnight => MiniGameKind::Heartbeat,
        }
    }
}

/// la mécanique propre à chaque mini-jeu. Pour en ajouter un : un fichier avec
/// new(rl, side) / update(rl, dt) -> Option<Attempt> / draw(d) / area(), puis une variante ici
enum Mechanic {
    Timing(TimingBar),
    Target(TargetShot),
    Charge(ChargeRing),
    Sequence(KeySequence),
    Runes(RuneMemory),
    Heartbeat(Heartbeat),
    Channel(Channel),
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
            MiniGameKind::Charge => Mechanic::Charge(ChargeRing::new(rl, side)),
            MiniGameKind::Sequence => Mechanic::Sequence(KeySequence::new(rl, side)),
            MiniGameKind::Runes => Mechanic::Runes(RuneMemory::new(rl, side)),
            MiniGameKind::Heartbeat => Mechanic::Heartbeat(Heartbeat::new(rl, side)),
            MiniGameKind::Channel => Mechanic::Channel(Channel::new(rl, side)),
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
            Mechanic::Charge(ring) => ring.update(rl, delta_time),
            Mechanic::Sequence(sequence) => sequence.update(rl, delta_time),
            Mechanic::Runes(runes) => runes.update(rl, delta_time),
            Mechanic::Heartbeat(heart) => heart.update(rl, delta_time),
            Mechanic::Channel(channel) => channel.update(rl, delta_time),
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

        // séquence, runes, cœur et canalisation gèrent leur temps eux-mêmes (trop lent = raté), le
        // chrono global ne doit pas les couper avant la fin
        let own_timer = matches!(
            self.mechanic,
            Mechanic::Sequence(_)
                | Mechanic::Runes(_)
                | Mechanic::Heartbeat(_)
                | Mechanic::Channel(_)
        );
        if !own_timer {
            self.time_remaining -= delta_time;
        }
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
            Mechanic::Charge(ring) => {
                ring.draw(d);
                ring.area()
            }
            Mechanic::Sequence(sequence) => {
                sequence.draw(d);
                sequence.area()
            }
            Mechanic::Runes(runes) => {
                runes.draw(d);
                runes.area()
            }
            Mechanic::Heartbeat(heart) => {
                heart.draw(d);
                heart.area()
            }
            Mechanic::Channel(channel) => {
                channel.draw(d);
                channel.area()
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

/// multiplicateur final d'un duel : attaque × parade (un soin n'a pas de parade)
pub fn duel_multiplier(attack: TimingResult, parry: Option<TimingResult>) -> f32 {
    attack.to_multiplier(true) * parry.map_or(1.0, |p| p.to_multiplier(false))
}

/// résultat du duel affiché au-dessus du combat
pub struct ResultDisplay {
    pub attack: TimingResult,
    pub parry: Option<TimingResult>,
    pub multiplier: f32,
    pub time_left: f32,
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
) -> Option<ResultDisplay> {
    if attacker.state != UnitState::MiniGame {
        return None;
    }
    let is_heal = matches!(attacker.pending_action, Some(PendingAction::Heal(_)));
    let player_is_attacker = attacker.faction == player_faction;

    // chaque camp joue le mini-jeu de SA classe avec SA barre. Le joueur joue toujours,
    // sauf quand l'IA soigne un de ses alliés : rien à contrer
    let player_result = if player_is_attacker || !is_heal {
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
        // on attend que le joueur ait fini
        Some(game.result?)
    } else {
        None
    };

    // le camp de l'IA ne joue pas : il tire son résultat selon les chances de sa barre
    let result_of = |is_attacker: bool| match player_result {
        Some(r) if is_attacker == player_is_attacker => r,
        _ => conditions
            .side(is_attacker)
            .odds()
            .pick(rl.get_random_value::<i32>(0..=999) as f32 / 1000.0),
    };
    let attack = result_of(true);
    let parry = (!is_heal).then(|| result_of(false));

    *damage_multiplier = duel_multiplier(attack, parry);
    attacker.state = if is_heal {
        UnitState::Healing
    } else {
        UnitState::Attacking
    };
    attacker_attack_animation.current = 0;
    attacker_attack_animation.finished = false;
    *minigame = None;
    Some(ResultDisplay {
        attack,
        parry,
        multiplier: *damage_multiplier,
        time_left: 1.0,
    })
}
