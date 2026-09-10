use crate::animation::Animation;
use crate::game_mode::GameMode;
use crate::unit::{Unit, UnitState};
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
    if attacker.state == UnitState::Attacking && !*attack_animation_started {
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
    attacker_attack_animation: &Animation,
    defender_hurt_animation: &mut Animation,
    attack_animation_started: &mut bool,
    damage_multiplier: f32,
) {
    if attacker.state == UnitState::Attacking && attacker_attack_animation.finished {
        defender.hp_points -= (attack_damage_dealt(attacker.attack_power, defender.defense) as f32
            * damage_multiplier) as i32;
        if defender.hp_points < 0 {
            defender.hp_points = 0;
        }
        defender_hurt_animation.current = 0;
        defender_hurt_animation.finished = false;
        defender.state = UnitState::Hurt;

        attacker.state = UnitState::Idle;
        attacker.attack_target = false;
        attacker.has_attacked = true;
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
