use raylib::prelude::*;

use crate::animation::Animation;
use crate::assets::Assets;
use crate::duel::{self, DuelConditions};
use crate::game::Game;
use crate::game_mode::GameMode;
use crate::minigame;
use crate::unit::{PendingAction, Unit, UnitState, two_mut};
use crate::{GRID_COLS, TILE_SIZE};

const SCREEN_WIDTH: f32 = (GRID_COLS * TILE_SIZE) as f32;

pub fn attack_damage_dealt(attack_power: i32, defense: i32) -> i32 {
    let reduction = defense as f32 / 100.0;
    let damage = attack_power as f32 * (1.0 - reduction);
    damage.max(0.0) as i32
}

pub fn start_attack_if_needed(
    attacker: &mut Unit,
    defender: &mut Unit,
    attacker_attack_animation: &mut Animation,
    attacker_attack_effect_animation: &mut Animation,
    attack_animation_started: &mut bool,
    game_mode: &mut GameMode,
    attacker_combat_x: &mut f32,
    defender_combat_x: &mut f32,
) {
    let is_acting = attacker.state == UnitState::Attacking || attacker.state == UnitState::Healing;
    if is_acting && !*attack_animation_started {
        attacker.state = UnitState::CombatEntering;
        defender.state = UnitState::CombatEntering;

        *game_mode = GameMode::CombatScreen;

        // reset motion
        attacker_attack_animation.current = 0;
        attacker_attack_animation.finished = false;

        // reset effect
        attacker_attack_effect_animation.current = 0;
        attacker_attack_effect_animation.finished = false;

        *attack_animation_started = true;

        if attacker.grid_x < defender.grid_x {
            attacker.facing_left = false;
            *attacker_combat_x = -200.0;
            defender.facing_left = true;
            *defender_combat_x = SCREEN_WIDTH + 200.0;
        } else {
            attacker.facing_left = true;
            *attacker_combat_x = SCREEN_WIDTH + 200.0;
            defender.facing_left = false;
            *defender_combat_x = -200.0;
        }
    }
}

pub fn enter_combat(
    attacker: &mut Unit,
    defender: &mut Unit,
    attacker_combat_x: &mut f32,
    defender_combat_x: &mut f32,
    attacker_target_x: f32,
    defender_target_x: f32,
    combat_entering_timer: &mut f32,
    delta_time: f32,
) {
    if attacker.state == UnitState::CombatEntering {
        let speed = 2.0;
        *attacker_combat_x += (attacker_target_x - *attacker_combat_x) * speed * delta_time;
        *defender_combat_x += (defender_target_x - *defender_combat_x) * speed * delta_time;

        *combat_entering_timer += delta_time;

        if *combat_entering_timer >= 1.5 {
            attacker.state = UnitState::Idle;
            defender.state = UnitState::Idle;
            *combat_entering_timer = 0.0;
        }
    }
}

pub fn start_attack_after_ready(
    attacker: &mut Unit,
    defender: &Unit,
    attack_in_progress: bool,
    ready_timer: &mut f32,
    delta_time: f32,
) {
    if !attack_in_progress {
        return;
    }

    if attacker.state == UnitState::Idle && defender.state == UnitState::Idle {
        *ready_timer += delta_time;
        if *ready_timer >= 0.3 {
            attacker.state = UnitState::MiniGame;
            *ready_timer = 0.0;
        }
    }
}

pub fn resolve_attack(
    attacker: &mut Unit,
    defender: &mut Unit,
    attacker_attack_finished: bool,
    defender_hurt_animation: &mut Animation,
    attack_animation_started: &mut bool,
    damage_multiplier: f32,
) {
    if attacker.state == UnitState::Attacking && attacker_attack_finished {
        defender.hp_points -= (attack_damage_dealt(attacker.attack_power, defender.defense) as f32
            * damage_multiplier) as i32;
        if defender.hp_points < 0 {
            defender.hp_points = 0;
        }
        defender_hurt_animation.current = 0;
        defender_hurt_animation.finished = false;
        defender.state = UnitState::Hurt;

        attacker.state = UnitState::Idle;
        attacker.pending_action = None;
        attacker.has_attacked = true;
        *attack_animation_started = false;
    }
}

pub fn resolve_heal(
    healer: &mut Unit,
    target: &mut Unit,
    heal_finished: bool,
    attack_animation_started: &mut bool,
    heal_multiplier: f32,
) {
    if healer.state == UnitState::Healing && heal_finished {
        let heal_amount = (healer.attack_power as f32 * heal_multiplier) as i32;
        target.hp_points = (target.hp_points + heal_amount).min(target.hp_max_points);
        target.state = UnitState::Idle;

        healer.state = UnitState::Idle;
        healer.pending_action = None;
        healer.has_attacked = true;
        *attack_animation_started = false;
    }
}

pub fn update_hurt_state(
    defender: &mut Unit,
    defender_hurt_animation: &Animation,
    defender_dying_animation: &mut Animation,
) {
    if defender.state == UnitState::Hurt {
        if defender_hurt_animation.finished {
            if defender.hp_points == 0 {
                defender_dying_animation.current = 0;
                defender_dying_animation.finished = false;
                defender.state = UnitState::Dying;
            } else {
                defender.state = UnitState::Idle;
            }
        }
    }
}

pub fn update_dying_state(defender: &mut Unit, defender_dying_animation: &Animation) {
    if defender.state == UnitState::Dying {
        if defender_dying_animation.finished {
            defender.state = UnitState::Dead;
        }
    }
}

pub fn combat_exit_pause_timer(
    defender: &mut Unit,
    attack_in_progress: bool,
    game_mode: &mut GameMode,
    combat_exit_pause_timer: &mut f32,
    delta_time: f32,
) {
    let combat_finished = !attack_in_progress
        && (defender.state == UnitState::Idle || defender.state == UnitState::Dead);

    if *game_mode == GameMode::CombatScreen && combat_finished {
        *combat_exit_pause_timer += delta_time;
        if *combat_exit_pause_timer >= 0.2 {
            *game_mode = GameMode::GridScreen;
            *combat_exit_pause_timer = 0.0;
        }
    }
}

/// déclenche le combat quand une unité passe en Attacking/Healing, puis le fait avancer :
/// entrée en scène, animation, minigame, dégâts/soin, hurt/dying, retour à la grille
pub fn update(game: &mut Game, assets: &mut Assets, rl: &RaylibHandle, delta_time: f32) {
    if game.game_mode == GameMode::GridScreen && game.active_combat.is_none() {
        if let Some(attacker_idx) = game
            .units
            .iter()
            .position(|u| u.state == UnitState::Attacking || u.state == UnitState::Healing)
        {
            if let Some(action) = game.units[attacker_idx].pending_action {
                let target_idx = action.target_idx();
                game.active_combat = Some((attacker_idx, target_idx));
                // le placement est figé maintenant : il décide des conditions du minigame
                game.duel_conditions = match action {
                    PendingAction::Attack(_) => {
                        let attacker = &game.units[attacker_idx];
                        duel::compute(
                            &game.units,
                            attacker_idx,
                            (attacker.grid_x, attacker.grid_y),
                            target_idx,
                        )
                    }
                    PendingAction::Heal(_) => DuelConditions::neutral(),
                };
            }
        }
    }

    let sprite_size = 500.0;
    let overlap = 185.0;
    let center_x = SCREEN_WIDTH / 2.0;
    let left_x = center_x - sprite_size + overlap;
    let right_x = center_x - overlap;

    // faire disparaître le message de résultat du minigame après un certain temps
    if let Some((_, time_left, _)) = &mut game.result_display {
        *time_left -= delta_time;
        if *time_left <= 0.0 {
            game.result_display = None;
        }
    }

    if let Some((attacker_idx, defender_idx)) = game.active_combat {
        let (attacker_target_x, defender_target_x) = if game.units[attacker_idx].facing_left {
            (right_x, left_x)
        } else {
            (left_x, right_x)
        };

        let is_heal = matches!(
            game.units[attacker_idx].pending_action,
            Some(PendingAction::Heal(_))
        );
        let attacker_class = game.units[attacker_idx].class;
        let defender_class = game.units[defender_idx].class;
        let (attacker, defender) = two_mut(&mut game.units, attacker_idx, defender_idx);

        {
            let (cast_anim, cast_effect) = if is_heal {
                assets.heal_animation_mut(attacker_class)
            } else {
                let set = assets.animation_set_mut(attacker_class);
                (&mut set.attack, &mut set.attack_effect)
            };
            start_attack_if_needed(
                attacker,
                defender,
                cast_anim,
                cast_effect,
                &mut game.attack_animation_started,
                &mut game.game_mode,
                &mut game.attacker_combat_x,
                &mut game.defender_combat_x,
            );
        }
        enter_combat(
            attacker,
            defender,
            &mut game.attacker_combat_x,
            &mut game.defender_combat_x,
            attacker_target_x,
            defender_target_x,
            &mut game.combat_entering_timer,
            delta_time,
        );
        start_attack_after_ready(
            attacker,
            defender,
            game.attack_animation_started,
            &mut game.combat_ready_timer,
            delta_time,
        );
        {
            let (cast_anim, _) = if is_heal {
                assets.heal_animation_mut(attacker_class)
            } else {
                let set = assets.animation_set_mut(attacker_class);
                (&mut set.attack, &mut set.attack_effect)
            };
            if let Some(result) = minigame::handle_minigame(
                attacker,
                defender,
                game.player_faction,
                &mut game.timing_bar,
                &game.duel_conditions,
                &mut game.damage_multiplier,
                rl,
                delta_time,
                cast_anim,
            ) {
                game.result_display = Some((result, 1.0, game.damage_multiplier));
            }
        }
        if is_heal {
            let cast_finished = assets.heal_animation_mut(attacker_class).0.finished;
            resolve_heal(
                attacker,
                defender,
                cast_finished,
                &mut game.attack_animation_started,
                game.damage_multiplier,
            );
        } else {
            let cast_finished = assets.animation_set_mut(attacker_class).attack.finished;
            let defender_hurt = &mut assets.animation_set_mut(defender_class).hurt;
            resolve_attack(
                attacker,
                defender,
                cast_finished,
                defender_hurt,
                &mut game.attack_animation_started,
                game.damage_multiplier,
            );
        }
        {
            let defender_set = assets.animation_set_mut(defender_class);
            update_hurt_state(defender, &defender_set.hurt, &mut defender_set.die);
            update_dying_state(defender, &defender_set.die);
        }
        combat_exit_pause_timer(
            defender,
            game.attack_animation_started,
            &mut game.game_mode,
            &mut game.combat_exit_pause_timer,
            delta_time,
        );
    }
}
