use crate::combat::attack_damage_dealt;
use crate::unit::Unit;

// valeurs de base de la barre de timing, modifiées ensuite par le placement sur la grille
pub const BASE_ZONE: f32 = 0.2;
pub const BASE_PERFECT: f32 = 0.03;
const MIN_ZONE: f32 = 0.06;
const MAX_ZONE: f32 = 0.45;
const MAX_LIVES: u8 = 2;

const FLANK_ZONE: f32 = 0.04; // par allié qui encercle (max 3)
const GUARD_ZONE: f32 = 0.03; // par allié qui garde le défenseur (max 2)
const PERFECT_BONUS: f32 = 0.03;
const SHOOTER_IN_MELEE_ZONE: f32 = 0.04;

/// conditions de la barre de timing pour un camp du duel
#[derive(Clone, Copy, Debug)]
pub struct DuelSide {
    pub zone_width: f32,
    pub perfect_width: f32,
    pub speed: f32,
    pub lives: u8,
}

impl DuelSide {
    fn base() -> DuelSide {
        DuelSide {
            zone_width: BASE_ZONE,
            perfect_width: BASE_PERFECT,
            speed: 1.0,
            lives: 0,
        }
    }

    fn clamped(mut self) -> DuelSide {
        self.zone_width = self.zone_width.clamp(MIN_ZONE, MAX_ZONE);
        self.perfect_width = self.perfect_width.clamp(0.0, self.zone_width);
        self.lives = self.lives.min(MAX_LIVES);
        self
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
    // dégâts avant le multiplicateur du minigame
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

    /// avantage net de l'attaquant (facteurs pour - facteurs contre), pour noter les cases
    pub fn score(&self) -> i32 {
        self.factors
            .iter()
            .map(|f| if f.favors_attacker { 1 } else { -1 })
            .sum()
    }
}

fn distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

fn pct(value: f32) -> i32 {
    (value * 100.0).round() as i32
}

fn is_desperate(unit: &Unit) -> bool {
    unit.hp_points * 10 <= unit.hp_max_points * 3
}

/// conditions du duel si `attacker_idx` attaque `defender_idx` depuis la case `from`.
/// `from` peut différer de la position actuelle de l'attaquant (preview avant déplacement)
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
        .min(3) as f32;
    if flankers > 0.0 {
        att.zone_width += FLANK_ZONE * flankers;
        def.zone_width -= FLANK_ZONE * flankers;
        factors.push(Factor {
            label: format!(
                "Flank x{}: zone +{}% / their zone -{}%",
                flankers as i32,
                pct(FLANK_ZONE * flankers),
                pct(FLANK_ZONE * flankers)
            ),
            favors_attacker: true,
        });
    }

    // revers : un allié pile de l'autre côté du défenseur, en mêlée
    if dist == 1 {
        let mirror = (2 * defender_pos.0 - from.0, 2 * defender_pos.1 - from.1);
        if others(attacker.faction).any(|(_, u)| (u.grid_x, u.grid_y) == mirror) {
            att.perfect_width += PERFECT_BONUS;
            def.perfect_width = 0.0;
            factors.push(Factor {
                label: format!(
                    "Pincer: perfect +{}% / no perfect parry",
                    pct(PERFECT_BONUS)
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
        att.zone_width -= GUARD_ZONE * guards as f32;
        def.lives += guards;
        factors.push(Factor {
            label: format!(
                "Guarded x{}: zone -{}% / they get +{} lives",
                guards,
                pct(GUARD_ZONE * guards as f32),
                guards
            ),
            favors_attacker: false,
        });
    }

    // tireurs : à distance on vise posé, au contact c'est la panique
    if attacker.attack_range >= 2 {
        if dist >= 2 {
            att.speed *= 0.8;
            factors.push(Factor {
                label: String::from("Aimed shot: cursor -20% speed"),
                favors_attacker: true,
            });
        } else {
            att.zone_width -= SHOOTER_IN_MELEE_ZONE;
            factors.push(Factor {
                label: format!("Point blank: zone -{}%", pct(SHOOTER_IN_MELEE_ZONE)),
                favors_attacker: false,
            });
        }
    }
    if defender.attack_range >= 2 && dist == 1 {
        def.zone_width -= SHOOTER_IN_MELEE_ZONE;
        def.speed *= 1.3;
        factors.push(Factor {
            label: format!(
                "Caught in melee: their zone -{}%, cursor +30%",
                pct(SHOOTER_IN_MELEE_ZONE)
            ),
            favors_attacker: true,
        });
    }

    // désespoir : sous 30% PV, la fenêtre parfaite s'élargit (comeback)
    if is_desperate(attacker) {
        att.perfect_width += PERFECT_BONUS;
        factors.push(Factor {
            label: format!("Desperate: perfect +{}%", pct(PERFECT_BONUS)),
            favors_attacker: true,
        });
    }
    if is_desperate(defender) && def.perfect_width > 0.0 {
        def.perfect_width += PERFECT_BONUS;
        factors.push(Factor {
            label: format!("They're desperate: their perfect +{}%", pct(PERFECT_BONUS)),
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
        assert_eq!(c.attacker.zone_width, BASE_ZONE);
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
        assert!(c.attacker.zone_width > BASE_ZONE);
        assert!(c.attacker.perfect_width > BASE_PERFECT);
        assert_eq!(c.defender.perfect_width, 0.0);
        assert_eq!(c.score(), 2);
    }

    #[test]
    fn garde_retrecit_la_zone_et_donne_des_vies() {
        let units = vec![
            unit(Faction::Human, 0, 0, 1),
            unit(Faction::Undead, 1, 0, 1),
            unit(Faction::Undead, 1, 1, 1),
        ];
        let c = compute(&units, 0, (0, 0), 1);
        assert!(c.attacker.zone_width < BASE_ZONE);
        assert_eq!(c.defender.lives, 1);
        assert_eq!(c.score(), -1);
    }

    #[test]
    fn preview_depuis_une_autre_case() {
        // l'archer est loin mais on simule l'attaque depuis (3, 0) : tir visé
        let units = vec![
            unit(Faction::Human, 9, 9, 3),
            unit(Faction::Undead, 0, 0, 1),
        ];
        let c = compute(&units, 0, (3, 0), 1);
        assert!(c.attacker.speed < 1.0);
    }
}
