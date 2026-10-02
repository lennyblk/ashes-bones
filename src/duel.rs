use crate::combat::attack_damage_dealt;
use crate::unit::Unit;

pub const BASE_WINDOW: f32 = 0.2;
pub const BASE_PRECISION: f32 = 0.03;
pub const MIN_WINDOW: f32 = 0.06;
pub const MAX_WINDOW: f32 = 0.45;
pub const MAX_LIVES: u8 = 2;

pub const FLANK_WINDOW: f32 = 0.04;
pub const MAX_FLANKERS: usize = 3;
pub const GUARD_WINDOW: f32 = 0.03;
pub const PRECISION_BONUS: f32 = 0.03;
pub const SHOOTER_IN_MELEE_WINDOW: f32 = 0.04;
pub const AIMED_TEMPO: f32 = 0.8;
pub const MELEE_PANIC_TEMPO: f32 = 1.3;
pub const DESPERATE_HP_PCT: i32 = 30;

#[derive(Clone, Copy, Debug)]
pub struct DuelSide {
    pub window: f32,
    pub precision: f32,
    pub tempo: f32,
    pub lives: u8,
}

impl DuelSide {
    fn base() -> DuelSide {
        DuelSide {
            window: BASE_WINDOW,
            precision: BASE_PRECISION,
            tempo: 1.0,
            lives: 0,
        }
    }

    fn clamped(mut self) -> DuelSide {
        self.window = self.window.clamp(MIN_WINDOW, MAX_WINDOW);
        self.precision = self.precision.clamp(0.0, self.window);
        self.lives = self.lives.min(MAX_LIVES);
        self
    }

    /// chances estimées de PERFECT / GOOD / BAD avec cette barre. Estimation grossière
    /// (zone large et lente = plus facile, chaque vie redonne un essai) : sert à comparer
    /// des cases entre elles, pas à prédire le vrai taux de réussite d'un joueur
    pub fn odds(&self) -> Odds {
        let single_try = (self.window / self.tempo * ODDS_SCALE).clamp(0.0, 0.95);
        let hit = 1.0 - (1.0 - single_try).powi(self.lives as i32 + 1);
        let perfect = hit * (2.0 * self.precision / self.window).min(1.0);
        Odds {
            perfect,
            good: hit - perfect,
            bad: 1.0 - hit,
        }
    }
}

/// réglage de l'estimation : window 20% -> 50% de réussite en un essai
const ODDS_SCALE: f32 = 2.5;

#[derive(Clone, Copy, Debug)]
pub struct Odds {
    pub perfect: f32,
    pub good: f32,
    pub bad: f32,
}

impl Odds {
    /// multiplicateur de dégâts infligés attendu en attaque (BAD x0.5 / GOOD x1 / PERFECT x1.5)
    pub fn attack_multiplier(&self) -> f32 {
        self.perfect * 1.5 + self.good + self.bad * 0.5
    }

    /// multiplicateur de dégâts subis attendu en défense (BAD x1.5 / GOOD x1 / PERFECT x0.5)
    pub fn parry_multiplier(&self) -> f32 {
        self.perfect * 0.5 + self.good + self.bad * 1.5
    }
}

/// une raison qui change le duel, affichée au joueur (preview sur la grille + écran de combat)
#[derive(Clone, Debug)]
pub struct Factor {
    pub label: String,
    pub favors_attacker: bool,
}

#[derive(Clone, Debug)]
pub struct DuelConditions {
    pub attacker: DuelSide,
    pub defender: DuelSide,
    pub factors: Vec<Factor>,
    pub base_damage: i32,
}

impl DuelConditions {
    /// soin ou cas sans placement : barre standard
    pub fn neutral() -> DuelConditions {
        DuelConditions {
            attacker: DuelSide::base(),
            defender: DuelSide::base(),
            factors: Vec::new(),
            base_damage: 0,
        }
    }

    /// la barre que joue un camp
    pub fn side(&self, is_attacker: bool) -> &DuelSide {
        if is_attacker {
            &self.attacker
        } else {
            &self.defender
        }
    }

    /// dégâts attendus de l'attaquant en % par rapport à un duel sans placement
    /// (ex : +12 = 12% de dégâts en plus en moyenne). Note des cases d'attaque
    pub fn advantage(&self) -> i32 {
        let neutral = DuelSide::base().odds().attack_multiplier();
        let here = self.attacker.odds().attack_multiplier();
        ((here / neutral - 1.0) * 100.0).round() as i32
    }
}

fn distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

fn pct(value: f32) -> i32 {
    (value * 100.0).round() as i32
}

fn is_desperate(unit: &Unit) -> bool {
    unit.hp_points * 100 <= unit.hp_max_points * DESPERATE_HP_PCT
}

pub fn compute(
    units: &[Unit],
    attacker_idx: usize,
    from: (i32, i32),
    defender_idx: usize,
) -> DuelConditions {
    let attacker = &units[attacker_idx];
    let defender = &units[defender_idx];
    let defender_pos = (defender.grid_x, defender.grid_y);
    let dist = distance(from, defender_pos);

    let mut att = DuelSide::base();
    let mut def = DuelSide::base();
    let mut factors = Vec::new();

    let others = |faction| {
        units.iter().enumerate().filter(move |(i, u)| {
            *i != attacker_idx && *i != defender_idx && u.is_alive() && u.faction == faction
        })
    };

    // encerclement : alliés de l'attaquant collés au défenseur
    let flankers = others(attacker.faction)
        .filter(|(_, u)| distance((u.grid_x, u.grid_y), defender_pos) == 1)
        .count()
        .min(MAX_FLANKERS) as f32;
    if flankers > 0.0 {
        att.window += FLANK_WINDOW * flankers;
        def.window -= FLANK_WINDOW * flankers;
        factors.push(Factor {
            label: format!(
                "Flank x{}: window +{}% / their window -{}%",
                flankers as i32,
                pct(FLANK_WINDOW * flankers),
                pct(FLANK_WINDOW * flankers)
            ),
            favors_attacker: true,
        });
    }

    // revers : un allié pile de l'autre côté du défenseur, en mêlée
    if dist == 1 {
        let mirror = (2 * defender_pos.0 - from.0, 2 * defender_pos.1 - from.1);
        if others(attacker.faction).any(|(_, u)| (u.grid_x, u.grid_y) == mirror) {
            att.precision += PRECISION_BONUS;
            def.precision = 0.0;
            factors.push(Factor {
                label: format!(
                    "Pincer: precision +{}% / no perfect parry",
                    pct(PRECISION_BONUS)
                ),
                favors_attacker: true,
            });
        }
    }

    // soutien : alliés collés à la case de l'attaquant
    let supporters = others(attacker.faction)
        .filter(|(_, u)| distance((u.grid_x, u.grid_y), from) == 1)
        .count()
        .min(MAX_LIVES as usize) as u8;
    if supporters > 0 {
        att.lives += supporters;
        factors.push(Factor {
            label: format!("Support x{}: +{} lives", supporters, supporters),
            favors_attacker: true,
        });
    }

    // garde : alliés du défenseur collés à lui
    let guards = others(defender.faction)
        .filter(|(_, u)| distance((u.grid_x, u.grid_y), defender_pos) == 1)
        .count()
        .min(MAX_LIVES as usize) as u8;
    if guards > 0 {
        att.window -= GUARD_WINDOW * guards as f32;
        def.lives += guards;
        factors.push(Factor {
            label: format!(
                "Guarded x{}: window -{}% / they get +{} lives",
                guards,
                pct(GUARD_WINDOW * guards as f32),
                guards
            ),
            favors_attacker: false,
        });
    }

    // tireurs : à distance on vise posé, au contact c'est la panique
    if attacker.attack_range >= 2 {
        if dist >= 2 {
            att.tempo *= AIMED_TEMPO;
            factors.push(Factor {
                label: format!("Aimed shot: tempo -{}%", pct(1.0 - AIMED_TEMPO)),
                favors_attacker: true,
            });
        } else {
            att.window -= SHOOTER_IN_MELEE_WINDOW;
            factors.push(Factor {
                label: format!("Point blank: window -{}%", pct(SHOOTER_IN_MELEE_WINDOW)),
                favors_attacker: false,
            });
        }
    }
    if defender.attack_range >= 2 && dist == 1 {
        def.window -= SHOOTER_IN_MELEE_WINDOW;
        def.tempo *= MELEE_PANIC_TEMPO;
        factors.push(Factor {
            label: format!(
                "Caught in melee: their window -{}%, tempo +{}%",
                pct(SHOOTER_IN_MELEE_WINDOW),
                pct(MELEE_PANIC_TEMPO - 1.0)
            ),
            favors_attacker: true,
        });
    }

    // désespoir : sous 30% PV, la fenêtre parfaite s'élargit (comeback)
    if is_desperate(attacker) {
        att.precision += PRECISION_BONUS;
        factors.push(Factor {
            label: format!("Desperate: precision +{}%", pct(PRECISION_BONUS)),
            favors_attacker: true,
        });
    }
    if is_desperate(defender) && def.precision > 0.0 {
        def.precision += PRECISION_BONUS;
        factors.push(Factor {
            label: format!(
                "They're desperate: their precision +{}%",
                pct(PRECISION_BONUS)
            ),
            favors_attacker: false,
        });
    }

    DuelConditions {
        attacker: att.clamped(),
        defender: def.clamped(),
        factors,
        base_damage: attack_damage_dealt(attacker.attack_power, defender.defense),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit::{Faction, UnitClass, UnitState};

    fn unit(faction: Faction, x: i32, y: i32, range: i32) -> Unit {
        Unit {
            name: String::from("u"),
            faction,
            class: UnitClass::Soldier,
            grid_x: x,
            grid_y: y,
            screen_x: 0.0,
            screen_y: 0.0,
            move_points: 2,
            move_points_remaining: 2,
            has_attacked: false,
            hp_points: 100,
            hp_max_points: 100,
            path: Vec::new(),
            state: UnitState::Idle,
            facing_left: false,
            attack_range: range,
            attack_power: 50,
            defense: 10,
            can_heal: false,
            pending_action: None,
        }
    }

    #[test]
    fn duel_seul_est_neutre() {
        let units = vec![
            unit(Faction::Human, 0, 0, 1),
            unit(Faction::Undead, 1, 0, 1),
        ];
        let c = compute(&units, 0, (0, 0), 1);
        assert!(c.factors.is_empty());
        assert_eq!(c.attacker.window, BASE_WINDOW);
        assert_eq!(c.base_damage, 45);
    }

    #[test]
    fn revers_donne_encerclement_et_supprime_la_parade_parfaite() {
        // attaquant à gauche, allié pile en face à droite du défenseur
        let units = vec![
            unit(Faction::Human, 0, 0, 1),
            unit(Faction::Undead, 1, 0, 1),
            unit(Faction::Human, 2, 0, 1),
        ];
        let c = compute(&units, 0, (0, 0), 1);
        assert!(c.attacker.window > BASE_WINDOW);
        assert!(c.attacker.precision > BASE_PRECISION);
        assert_eq!(c.defender.precision, 0.0);
        assert!(c.advantage() > 0);
    }

    #[test]
    fn garde_retrecit_la_zone_et_donne_des_vies() {
        let units = vec![
            unit(Faction::Human, 0, 0, 1),
            unit(Faction::Undead, 1, 0, 1),
            unit(Faction::Undead, 1, 1, 1),
        ];
        let c = compute(&units, 0, (0, 0), 1);
        assert!(c.attacker.window < BASE_WINDOW);
        assert_eq!(c.defender.lives, 1);
        assert!(c.advantage() < 0);
    }

    #[test]
    fn preview_depuis_une_autre_case() {
        // l'archer est loin mais on simule l'attaque depuis (3, 0) : tir visé
        let units = vec![
            unit(Faction::Human, 9, 9, 3),
            unit(Faction::Undead, 0, 0, 1),
        ];
        let c = compute(&units, 0, (3, 0), 1);
        assert!(c.attacker.tempo < 1.0);
    }
}
