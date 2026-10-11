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
use crate::settings::{self, Bind, tr};
use crate::trf;
use crate::unit::{Faction, PendingAction, Unit, UnitClass, UnitState};
use raylib::prelude::*;

use channel::Channel;
use charge::ChargeRing;
use heartbeat::Heartbeat;
use runes::RuneMemory;
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

    pub const ALL: [MiniGameKind; 7] = [
        MiniGameKind::Timing,
        MiniGameKind::Charge,
        MiniGameKind::Target,
        MiniGameKind::Sequence,
        MiniGameKind::Runes,
        MiniGameKind::Heartbeat,
        MiniGameKind::Channel,
    ];

    /// les classes qui jouent ce mini-jeu
    pub fn classes(self) -> &'static [UnitClass] {
        match self {
            MiniGameKind::Timing => &[UnitClass::Soldier, UnitClass::Skeleton],
            MiniGameKind::Target => &[UnitClass::Longbowman],
            MiniGameKind::Charge => &[UnitClass::Cavalry, UnitClass::Ghoul],
            MiniGameKind::Sequence => &[UnitClass::Assassin, UnitClass::Wraith],
            MiniGameKind::Runes => &[UnitClass::Mage, UnitClass::Banshee],
            MiniGameKind::Heartbeat => &[UnitClass::BloodKnight],
            MiniGameKind::Channel => &[UnitClass::Priest, UnitClass::Necromancer],
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            MiniGameKind::Timing => tr("Strike", "Frappe"),
            MiniGameKind::Target => tr("Aim", "Visée"),
            MiniGameKind::Charge => tr("Charge", "Charge"),
            MiniGameKind::Sequence => tr("Combo", "Combo"),
            MiniGameKind::Runes => tr("Runes", "Runes"),
            MiniGameKind::Heartbeat => tr("Blood pulse", "Pouls de sang"),
            MiniGameKind::Channel => tr("Channel", "Canalisation"),
        }
    }

    /// la règle en deux lignes courtes (guide et entraînement)
    pub fn rule(self) -> [&'static str; 2] {
        match self {
            MiniGameKind::Timing => [
                tr("Stop the cursor", "Arrête le curseur"),
                tr("in the yellow zone.", "dans la zone jaune."),
            ],
            MiniGameKind::Target => [
                tr("Shoot the moving", "Tire sur la cible"),
                tr("target.", "qui bouge."),
            ],
            MiniGameKind::Charge => [
                tr("Strike when the ring", "Frappe quand l'anneau"),
                tr("crosses the yellow band.", "passe sur le jaune."),
            ],
            MiniGameKind::Sequence => [
                tr("Type the 4 arrows", "Tape les 4 flèches"),
                tr("in order, fast!", "dans l'ordre, vite !"),
            ],
            MiniGameKind::Runes => [
                tr("Watch the runes,", "Regarde les runes,"),
                tr("then repeat them.", "puis refais la suite."),
            ],
            MiniGameKind::Heartbeat => [
                tr("Hit the 3 drops", "Frappe les 3 gouttes"),
                tr("on the beat.", "en rythme."),
            ],
            MiniGameKind::Channel => [
                tr("Keep your cursor", "Garde ton curseur"),
                tr("in the drifting aura.", "dans l'aura qui dérive."),
            ],
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
                runes.draw_key_hints(d, &assets.info_font);
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
        self.draw_hint(d, assets, area);
    }

    /// comment jouer, en une ligne au-dessus du mini-jeu (avec les touches choisies)
    pub fn hint(&self) -> String {
        let action = settings::key_name(settings::key(Bind::Action));
        match &self.mechanic {
            Mechanic::Timing(_) => {
                trf!("Click or {} in the yellow zone", action ; "Clic ou {} dans la zone jaune", action)
            }
            Mechanic::Target(_) => {
                trf!("Aim, then click or {}", action ; "Vise, puis clic ou {}", action)
            }
            Mechanic::Charge(_) => trf!(
                "Click or {} when the ring hits yellow", action ;
                "Clic ou {} quand l'anneau touche le jaune", action
            ),
            Mechanic::Sequence(_) => {
                let keys: Vec<String> = [Bind::Up, Bind::Left, Bind::Down, Bind::Right]
                    .iter()
                    .map(|b| settings::key_name(settings::key(*b)))
                    .collect();
                let keys = keys.join(" ");
                trf!("Arrows or {}, in order", keys ; "Flèches ou {}, dans l'ordre", keys)
            }
            Mechanic::Runes(_) => tr(
                "Watch, then repeat (click or keys)",
                "Regarde, puis refais (clic ou touches)",
            )
            .to_string(),
            Mechanic::Heartbeat(_) => {
                trf!("Click or {} on each drop", action ; "Clic ou {} sur chaque goutte", action)
            }
            Mechanic::Channel(_) => tr(
                "Keep your cursor in the aura",
                "Garde le curseur dans l'aura",
            )
            .to_string(),
        }
    }

    fn draw_hint(&self, d: &mut RaylibDrawHandle, assets: &Assets, area: Rectangle) {
        let text = self.hint();
        let size = assets.info_font.measure_text(&text, 20.0, 1.0);
        let pos = Vector2::new(SCREEN_WIDTH as f32 / 2.0 - size.x / 2.0, area.y - 78.0);
        d.draw_rectangle_rounded(
            Rectangle::new(pos.x - 12.0, pos.y - 5.0, size.x + 24.0, size.y + 10.0),
            0.5,
            8,
            Color::new(10, 10, 14, 190),
        );
        d.draw_text_ex(
            &assets.info_font,
            &text,
            pos,
            20.0,
            1.0,
            Color::new(235, 228, 210, 255),
        );
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
            let text = tr("SAVED!", "SAUVÉ !");
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

/// online : échange des résultats du duel en cours avec l'adversaire
#[derive(Default)]
pub struct DuelExchange {
    pub mine: Option<TimingResult>,
    pub to_send: Option<TimingResult>,
    pub theirs: Option<TimingResult>,
}

pub fn handle_minigame(
    attacker: &mut Unit,
    defender: &mut Unit,
    player_faction: Faction,
    online: bool,
    exchange: &mut DuelExchange,
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
    // je joue toujours, sauf quand c'est l'adversaire qui soigne : rien à contrer
    let player_plays = player_is_attacker || !is_heal;
    // l'adversaire a un résultat, sauf quand c'est moi qui soigne
    let opponent_plays = !player_is_attacker || !is_heal;

    // 1) mon mini-jeu (celui de MA classe, avec MA barre) jusqu'à avoir un résultat
    if player_plays && exchange.mine.is_none() {
        let game = minigame.get_or_insert_with(|| {
            let class = if player_is_attacker {
                attacker.class
            } else {
                defender.class
            };
            MiniGame::new(
                MiniGameKind::for_class(class),
                conditions.side(player_is_attacker),
                rl,
            )
        });
        game.update(rl, delta_time);
        let Some(result) = game.result else {
            return None; // pas encore fini
        };
        exchange.mine = Some(result);
        *minigame = None;
        if online {
            exchange.to_send = Some(result);
        }
    }

    // 2) le résultat de l'autre : reçu par le réseau en ligne, tiré selon ses chances contre l'IA
    let theirs = if !opponent_plays {
        None
    } else if online {
        let Some(result) = exchange.theirs.take() else {
            return None; // on attend le sien
        };
        Some(result)
    } else {
        let roll = rl.get_random_value::<i32>(0..=999) as f32 / 1000.0;
        Some(conditions.side(!player_is_attacker).odds().pick(roll))
    };

    // 3) les deux sont là : multiplicateur, puis animation (les dégâts tombent à sa fin)
    let mine = exchange.mine.take();
    let (attack, parry) = if player_is_attacker {
        (mine, theirs)
    } else {
        (theirs, mine)
    };
    let attack = attack.expect("l'attaquant ou le soigneur a toujours un résultat");

    *damage_multiplier = duel_multiplier(attack, parry);
    attacker.state = if is_heal {
        UnitState::Healing
    } else {
        UnitState::Attacking
    };
    attacker_attack_animation.current = 0;
    attacker_attack_animation.finished = false;
    Some(ResultDisplay {
        attack,
        parry,
        multiplier: *damage_multiplier,
        time_left: 1.0,
    })
}
