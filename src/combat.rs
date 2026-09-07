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
    human: &mut Human,
    enemy: &mut Undead,
    human_attack_animation: &mut Animation,
    human_attack_effect_animation: &mut Animation,
    attack_animation_started: &mut bool,
    game_mode: &mut GameMode,
    human_combat_x: &mut f32,
    enemy_combat_x: &mut f32,
) {
    if human.state == HumanState::Combat && !*attack_animation_started {
        human.state = HumanState::CombatEntering;
        enemy.state = UndeadState::CombatEntering;

        *game_mode = GameMode::CombatScreen;

        // reset motion
        human_attack_animation.current = 0;
        human_attack_animation.finished = false;

        // reset effect
        human_attack_effect_animation.current = 0;
        human_attack_effect_animation.finished = false;

        *attack_animation_started = true;

        if human.grid_x < enemy.grid_x {
            human.facing_left = false;
            *human_combat_x = -200.0;
            enemy.facing_left = true;
            *enemy_combat_x = SCREEN_WIDTH + 200.0;
        } else {
            human.facing_left = true;
            *human_combat_x = SCREEN_WIDTH + 200.0;
            enemy.facing_left = false;
            *enemy_combat_x = -200.0;
        }
    }
}

pub fn enter_combat(
    human: &mut Human,
    enemy: &mut Undead,
    human_combat_x: &mut f32,
    enemy_combat_x: &mut f32,
    human_target_x: f32,
    enemy_target_x: f32,
    combat_entering_timer: &mut f32,
    delta_time: f32,
    human_attack_animation: &mut Animation,
) {
    if human.state == HumanState::CombatEntering {
        let speed = 2.0;
        *human_combat_x += (human_target_x - *human_combat_x) * speed * delta_time;
        *enemy_combat_x += (enemy_target_x - *enemy_combat_x) * speed * delta_time;

        *combat_entering_timer += delta_time;

        if *combat_entering_timer >= 1.5 {
            human.state = HumanState::Combat;
            enemy.state = UndeadState::CombatEntering;
            human_attack_animation.current = 0;
            human_attack_animation.finished = false;
            *combat_entering_timer = 0.0;
        }
    }
}

pub fn resolve_attack(
    human: &mut Human,
    enemy: &mut Undead,
    human_attack_animation: &Animation,
    enemy_hurt_animation: &mut Animation,
    attack_animation_started: &mut bool,
) {
    if human.state == HumanState::Combat && human_attack_animation.finished {
        enemy.hp_points -= attack_damage_dealt(human.attack_power, enemy.defense);
        if enemy.hp_points < 0 {
            enemy.hp_points = 0;
        }
        enemy_hurt_animation.current = 0;
        enemy_hurt_animation.finished = false;
        enemy.state = UndeadState::Hurt;

        human.state = HumanState::Idle;
        human.attack_target = false;
        *attack_animation_started = false;
    }
}

pub fn update_hurt_state(
    enemy: &mut Undead,
    delta_time: f32,
    enemy_hurt_animation: &mut Animation,
    enemy_dying_animation: &mut Animation,
) {
    if enemy.state == UndeadState::Hurt {
        enemy_hurt_animation.animation_update(delta_time);
        if enemy_hurt_animation.finished {
            if enemy.hp_points == 0 {
                enemy_dying_animation.current = 0;
                enemy_dying_animation.finished = false;
                enemy.state = UndeadState::Dying;
            } else {
                enemy.state = UndeadState::Idle;
            }
        }
    }
}

pub fn update_dying_state(
    enemy: &mut Undead,
    delta_time: f32,
    enemy_dying_animation: &mut Animation,
) {
    if enemy.state == UndeadState::Dying {
        enemy_dying_animation.animation_update(delta_time);
        if enemy_dying_animation.finished {
            enemy.state = UndeadState::Dead;
        }
    }
}

pub fn combat_exit_pause_timer(
    wraith: &mut Undead,
    game_mode: &mut GameMode,
    combat_exit_pause_timer: &mut f32,
    delta_time: f32,
) {
    let combat_finished = wraith.state == UndeadState::Idle || wraith.state == UndeadState::Dead;

    if *game_mode == GameMode::CombatScreen && combat_finished {
        *combat_exit_pause_timer += delta_time;
        if *combat_exit_pause_timer >= 0.5 {
            *game_mode = GameMode::GridScreen;
            *combat_exit_pause_timer = 0.0;
        }
    }
}
