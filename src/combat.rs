use crate::animation::Animation;
use crate::game_mode::GameMode;
use crate::human::{Human, HumanState};
use crate::undead::{Undead, UndeadState};

const TILE_SIZE: f32 = 48.0;
const GRID_COLS: f32 = 25.0;
const GRID_ROWS: f32 = 16.0;
const SCREEN_WIDTH: f32 = GRID_COLS * TILE_SIZE;
const SCREEN_HEIGHT: f32 = GRID_ROWS * TILE_SIZE;

pub fn attack_damage_dealt(attack_power: i32, defense: i32) -> i32 {
    let reduction = defense as f32 / 100.0;
    let damage = attack_power as f32 * (1.0 - reduction);
    damage.max(0.0) as i32
}

pub fn start_attack_if_needed(
    soldier: &mut Human,
    wraith: &mut Undead,
    soldier_attack_animation: &mut Animation,
    soldier_attack_effect_animation: &mut Animation,
    attack_animation_started: &mut bool,
    game_mode: &mut GameMode,
    soldier_combat_x: &mut f32,
    wraith_combat_x: &mut f32,
) {
    if soldier.state == HumanState::Combat && !*attack_animation_started {
        soldier.state = HumanState::CombatEntering;
        wraith.state = UndeadState::CombatEntering;

        *game_mode = GameMode::CombatScreen;

        // reset motion
        soldier_attack_animation.current = 0;
        soldier_attack_animation.finished = false;

        // reset effect
        soldier_attack_effect_animation.current = 0;
        soldier_attack_effect_animation.finished = false;

        *attack_animation_started = true;

        if soldier.grid_x < wraith.grid_x {
            soldier.facing_left = false;
            *soldier_combat_x = -200.0;
            wraith.facing_left = true;
            *wraith_combat_x = SCREEN_WIDTH + 200.0;
        } else {
            soldier.facing_left = true;
            *soldier_combat_x = SCREEN_WIDTH + 200.0;
            wraith.facing_left = false;
            *wraith_combat_x = -200.0;
        }
    }
}

pub fn enter_combat(
    soldier: &mut Human,
    wraith: &mut Undead,
    soldier_combat_x: &mut f32,
    wraith_combat_x: &mut f32,
    soldier_target_x: f32,
    wraith_target_x: f32,
    combat_entering_timer: &mut f32,
    delta_time: f32,
    soldier_attack_animation: &mut Animation,
) {
    if soldier.state == HumanState::CombatEntering {
        let speed = 2.0;
        *soldier_combat_x += (soldier_target_x - *soldier_combat_x) * speed * delta_time;
        *wraith_combat_x += (wraith_target_x - *wraith_combat_x) * speed * delta_time;

        *combat_entering_timer += delta_time;

        if *combat_entering_timer >= 1.5 {
            soldier.state = HumanState::Combat;
            wraith.state = UndeadState::CombatEntering;
            soldier_attack_animation.current = 0;
            soldier_attack_animation.finished = false;
            *combat_entering_timer = 0.0;
        }
    }
}

pub fn resolve_attack(
    soldier: &mut Human,
    wraith: &mut Undead,
    soldier_attack_animation: &Animation,
    wraith_hurt_animation: &mut Animation,
    attack_animation_started: &mut bool,
) {
    if soldier.state == HumanState::Combat && soldier_attack_animation.finished {
        wraith.hp_points -= attack_damage_dealt(soldier.attack_power, wraith.defense);
        if wraith.hp_points < 0 {
            wraith.hp_points = 0;
        }
        wraith_hurt_animation.current = 0;
        wraith_hurt_animation.finished = false;
        wraith.state = UndeadState::Hurt;

        soldier.state = HumanState::Idle;
        soldier.attack_target = false;
        *attack_animation_started = false;
        println!("Undead HP: {}", wraith.hp_points);
    }
}

pub fn update_hurt_state(
    wraith: &mut Undead,
    delta_time: f32,
    wraith_hurt_animation: &mut Animation,
    wraith_dying_animation: &mut Animation,
    game_mode: &mut GameMode,
) {
    if wraith.state == UndeadState::Hurt {
        wraith_hurt_animation.animation_update(delta_time);
        if wraith_hurt_animation.finished {
            if wraith.hp_points == 0 {
                wraith_dying_animation.current = 0;
                wraith_dying_animation.finished = false;
                wraith.state = UndeadState::Dying;
            } else {
                wraith.state = UndeadState::Idle;
                *game_mode = GameMode::GridScreen;
            }
        }
    }
}

pub fn update_dying_state(
    wraith: &mut Undead,
    delta_time: f32,
    wraith_dying_animation: &mut Animation,
    game_mode: &mut GameMode,
) {
    if wraith.state == UndeadState::Dying {
        wraith_dying_animation.animation_update(delta_time);
        if wraith_dying_animation.finished {
            wraith.state = UndeadState::Dead;
            *game_mode = GameMode::GridScreen;
        }
    }
}
