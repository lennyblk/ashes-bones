use crate::animation::Animation;
use crate::character::{Character, CharacterState};
use crate::enemy::{Enemy, EnemyState};
use crate::game_mode::GameMode;

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
    character: &mut Character,
    enemy: &mut Enemy,
    human_attack_animation: &mut Animation,
    attack_animation_started: &mut bool,
    game_mode: &mut GameMode,
    character_combat_x: &mut f32,
    enemy_combat_x: &mut f32,
) {
    if character.state == CharacterState::Combat && !*attack_animation_started {
        character.state = CharacterState::CombatEntering;
        enemy.state = EnemyState::CombatEntering;

        *game_mode = GameMode::CombatScreen;
        human_attack_animation.current = 0;
        human_attack_animation.finished = false;
        *attack_animation_started = true;

        if character.grid_x < enemy.grid_x {
            character.facing_left = false;
            *character_combat_x = -200.0;
            enemy.facing_left = true;
            *enemy_combat_x = SCREEN_WIDTH + 200.0;
        } else {
            character.facing_left = true;
            *character_combat_x = SCREEN_WIDTH + 200.0;
            enemy.facing_left = false;
            *enemy_combat_x = -200.0;
        }
    }
}

pub fn enter_combat(
    character: &mut Character,
    enemy: &mut Enemy,
    character_combat_x: &mut f32,
    enemy_combat_x: &mut f32,
    character_target_x: f32,
    enemy_target_x: f32,
    combat_entering_timer: &mut f32,
    delta_time: f32,
    human_attack_animation: &mut Animation,
) {
    if character.state == CharacterState::CombatEntering {
        let speed = 2.0;
        *character_combat_x += (character_target_x - *character_combat_x) * speed * delta_time;
        *enemy_combat_x += (enemy_target_x - *enemy_combat_x) * speed * delta_time;

        *combat_entering_timer += delta_time;

        if *combat_entering_timer >= 1.5 {
            character.state = CharacterState::Combat;
            enemy.state = EnemyState::CombatEntering;
            human_attack_animation.current = 0;
            human_attack_animation.finished = false;
            *combat_entering_timer = 0.0;
        }
    }
}

pub fn resolve_attack(
    character: &mut Character,
    enemy: &mut Enemy,
    human_attack_animation: &Animation,
    enemy_hurt_animation: &mut Animation,
    attack_animation_started: &mut bool,
) {
    if character.state == CharacterState::Combat && human_attack_animation.finished {
        enemy.hp_points -= attack_damage_dealt(character.attack_power, enemy.defense);
        if enemy.hp_points < 0 {
            enemy.hp_points = 0;
        }
        enemy_hurt_animation.current = 0;
        enemy_hurt_animation.finished = false;
        enemy.state = EnemyState::Hurt;

        character.state = CharacterState::Idle;
        character.attack_target = false;
        *attack_animation_started = false;
        println!("Enemy HP: {}", enemy.hp_points);
    }
}

pub fn update_hurt_state(
    enemy: &mut Enemy,
    delta_time: f32,
    enemy_hurt_animation: &mut Animation,
    enemy_dying_animation: &mut Animation,
    game_mode: &mut GameMode,
) {
    if enemy.state == EnemyState::Hurt {
        enemy_hurt_animation.animation_update(delta_time);
        if enemy_hurt_animation.finished {
            if enemy.hp_points == 0 {
                enemy_dying_animation.current = 0;
                enemy_dying_animation.finished = false;
                enemy.state = EnemyState::Dying;
            } else {
                enemy.state = EnemyState::Idle;
                *game_mode = GameMode::GridScreen;
            }
        }
    }
}

pub fn update_dying_state(
    enemy: &mut Enemy,
    delta_time: f32,
    enemy_dying_animation: &mut Animation,
    game_mode: &mut GameMode,
) {
    if enemy.state == EnemyState::Dying {
        enemy_dying_animation.animation_update(delta_time);
        if enemy_dying_animation.finished {
            enemy.state = EnemyState::Dead;
            *game_mode = GameMode::GridScreen;
        }
    }
}
