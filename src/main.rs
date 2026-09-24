use raylib::consts::MouseButton::*;
use raylib::prelude::*;
use std::collections::HashMap;
mod ai;
mod animation;
mod assets;
mod combat;
mod cursor;
mod game_mode;
mod input;
mod map;
mod minigame;
mod movement;
mod render;
mod ui;
mod unit;

use animation::Animation;
use cursor::{CursorType, Cursors};
use movement::MovementRange;
use unit::{Faction, Unit, UnitClass, UnitState, two_mut};

const TILE_SIZE: i32 = 48;
const GRID_COLS: i32 = 25;
const GRID_ROWS: i32 = 16;
const SCREEN_WIDTH: i32 = GRID_COLS * TILE_SIZE;
const SCREEN_HEIGHT: i32 = GRID_ROWS * TILE_SIZE;
const ENEMY_TURN_DELAY: f32 = 0.5;

fn main() {
    // init window
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Ashes&Bones")
        .build();
    rl.hide_cursor();

    let mut assets = assets::load_assets(&mut rl, &thread);

    fn grid_to_screen_x(grid_x: i32) -> f32 {
        let x = (grid_x * TILE_SIZE) as f32 + (-40.0);
        x
    }

    fn grid_to_screen_y(grid_y: i32) -> f32 {
        let y = (grid_y * TILE_SIZE) as f32 + (-40.0);
        y
    }

    let mouse_position = rl.get_mouse_position();

    let cursor_grid_x = mouse_position.x as i32 / TILE_SIZE;
    let cursor_grid_y = mouse_position.y as i32 / TILE_SIZE;

    let mut cursor = Cursors {
        cursor_type: CursorType::Normal,
        position: Vector2::new(
            (cursor_grid_x * TILE_SIZE) as f32,
            (cursor_grid_y * TILE_SIZE) as f32,
        ),
    };
    // mon index dans units (aucune -> None)
    let mut selected_unit: Option<usize> = None;

    let hud = ui::HudRects::new();

    let soldier = Unit {
        name: String::from("Soldier"),
        faction: Faction::Human,
        class: UnitClass::Soldier,
        grid_x: 6,
        grid_y: 10,
        screen_x: grid_to_screen_x(6),
        screen_y: grid_to_screen_y(10),
        move_points: 2,
        move_points_remaining: 2,
        has_attacked: false,
        hp_points: 100,
        hp_max_points: 100,
        path: Vec::new(),
        state: UnitState::Idle,
        facing_left: false,
        attack_range: 1,
        attack_power: 100,
        defense: 5,
        attack_target: false,
    };

    let wraith = Unit {
        name: String::from("Wraith"),
        faction: Faction::Undead,
        class: UnitClass::Wraith,
        grid_x: 5,
        grid_y: 10,
        screen_x: grid_to_screen_x(5),
        screen_y: grid_to_screen_y(10),
        move_points: 2,
        move_points_remaining: 2,
        has_attacked: false,
        hp_points: 100,
        hp_max_points: 100,
        path: Vec::new(),
        state: UnitState::Idle,
        facing_left: false,
        attack_range: 1,
        attack_power: 15,
        defense: 10,
        attack_target: false,
    };

    let cavalry = Unit {
        name: String::from("Cavalry"),
        faction: Faction::Human,
        class: UnitClass::Cavalry,
        grid_x: 7,
        grid_y: 10,
        screen_x: grid_to_screen_x(7),
        screen_y: grid_to_screen_y(10),
        move_points: 3,
        move_points_remaining: 3,
        has_attacked: false,
        hp_points: 90,
        hp_max_points: 90,
        path: Vec::new(),
        state: UnitState::Idle,
        facing_left: false,
        attack_range: 1,
        attack_power: 70,
        defense: 8,
        attack_target: false,
    };

    // units[0] = soldier, units[1] = wraith, units[2] = cavalry
    let mut units: Vec<Unit> = vec![soldier, wraith, cavalry];

    // état de départ, pour le bouton retry
    let units_initial = units.clone();

    let mut game_mode = game_mode::GameMode::TitleScreen;
    let mut attacker_combat_x: f32 = 0.0;
    let mut defender_combat_x: f32 = 0.0;
    let mut combat_entering_timer: f32 = 0.0;
    let mut combat_ready_timer: f32 = 0.0;
    let mut combat_exit_pause_timer: f32 = 0.0;
    let tile_map = map::TileMap::load(&mut rl, &thread, "assets/maps/ashes-bones-map.tmx");
    let blocked_tiles = tile_map.blocked_tiles.clone();

    let mut attack_animation_started = false;
    // (indice attaquant, indice défenseur) dans `units`
    let mut active_combat: Option<(usize, usize)> = None;

    let mut current_turn = game_mode::TurnPhase::PlayerTurn;
    let mut enemy_turn_delay: f32 = 0.0;

    let mut timing_bar: Option<minigame::TimingBar> = None;
    let mut damage_multiplier: f32 = 1.0;
    let mut player_faction = Faction::Human;
    let mut result_display: Option<(minigame::TimingResult, f32, f32)> = None; // alert qui pop au resultat du minigame

    // run window --------------------------------------------------------------
    while !rl.window_should_close() {
        let delta_time = rl.get_frame_time();

        let mouse_position = rl.get_mouse_position();
        fn mouse_is_clicked(rl: &RaylibHandle) -> bool {
            rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
        }
        units[0].update_position(delta_time);
        units[0].advance_path();

        units[1].update_position(delta_time);
        units[1].advance_path();

        units[2].update_position(delta_time);
        units[2].advance_path();

        let mut click_consumed = false;

        // title screen : play / settings / exit ------------------------------
        if game_mode == game_mode::GameMode::TitleScreen {
            let clicked = mouse_is_clicked(&rl);
            if input::is_button_clicked(mouse_position, clicked, hud.btn_exit_title_screen) {
                break;
            }
            if input::is_button_clicked(mouse_position, clicked, hud.btn_settings_title_screen) {
                // TODO: écran settings
            }
            if input::is_button_clicked(mouse_position, clicked, hud.btn_play_title_screen) {
                game_mode = game_mode::GameMode::FactionSelectionScreen;
                click_consumed = true;
            }
        }

        // Faction selection screen : Human / Undead --------------------------------
        if game_mode == game_mode::GameMode::FactionSelectionScreen {
            if input::cancel_pressed(&rl) {
                game_mode = game_mode::GameMode::TitleScreen;
            } else if !click_consumed && mouse_is_clicked(&rl) {
                player_faction = if mouse_position.x < SCREEN_WIDTH as f32 / 2.0 {
                    Faction::Human
                } else {
                    Faction::Undead
                };
                game_mode = game_mode::GameMode::GridScreen;
            }
        }

        // écran de fin : retry / back / exit --------------------------------
        let game_over =
            game_mode == game_mode::GameMode::Victory || game_mode == game_mode::GameMode::Defeat;
        if game_over {
            let clicked = mouse_is_clicked(&rl);
            if input::is_button_clicked(mouse_position, clicked, hud.btn_exit) {
                break;
            }
            if input::is_button_clicked(mouse_position, clicked, hud.btn_back) {
                units = units_initial.clone();
                game_mode = game_mode::GameMode::TitleScreen;
                current_turn = game_mode::TurnPhase::PlayerTurn;
                enemy_turn_delay = 0.0;
                active_combat = None;
                attack_animation_started = false;
                combat_entering_timer = 0.0;
                combat_ready_timer = 0.0;
                combat_exit_pause_timer = 0.0;
                selected_unit = None;
                click_consumed = true;
            }
            if input::is_button_clicked(mouse_position, clicked, hud.btn_retry) {
                units = units_initial.clone();
                game_mode = game_mode::GameMode::GridScreen;
                current_turn = game_mode::TurnPhase::PlayerTurn;
                enemy_turn_delay = 0.0;
                active_combat = None;
                attack_animation_started = false;
                combat_entering_timer = 0.0;
                combat_ready_timer = 0.0;
                combat_exit_pause_timer = 0.0;
                selected_unit = None;
                click_consumed = true;
            }
        }
        let player_can_act = current_turn == game_mode::TurnPhase::PlayerTurn
            && game_mode == game_mode::GameMode::GridScreen
            && units[0].state == UnitState::Idle;

        // bouton wait
        let wait_button_visible =
            player_can_act && selected_unit.is_some_and(|i| units[i].can_wait());
        if wait_button_visible
            && input::is_button_clicked(mouse_position, mouse_is_clicked(&rl), hud.btn_wait)
        {
            if let Some(sel) = selected_unit {
                units[sel].wait();
            }
            selected_unit = None;
            click_consumed = true;
        }

        // fin de tour : bouton end turn, ou auto quand tous les persos ont fini
        let end_turn_clicked =
            input::is_button_clicked(mouse_position, mouse_is_clicked(&rl), hud.btn_end_turn);

        let all_units_finished = units[0].has_finished_turn(&[units[1].clone()]);

        if player_can_act && (end_turn_clicked || all_units_finished) {
            current_turn = game_mode::TurnPhase::EnemyTurn;
            enemy_turn_delay = ENEMY_TURN_DELAY;
            selected_unit = None;
            click_consumed = true;
        }

        if current_turn == game_mode::TurnPhase::EnemyTurn && enemy_turn_delay > 0.0 {
            enemy_turn_delay -= delta_time;
            if enemy_turn_delay <= 0.0 {
                enemy_turn_delay = 0.0;
                let human_units: Vec<Unit> = units
                    .iter()
                    .filter(|u| u.faction == Faction::Human && u.is_alive())
                    .cloned()
                    .collect();
                ai::take_turn(&mut units[1], &human_units, &blocked_tiles);
            }
        }

        // combat -----------------------------------------------------------
        if game_mode == game_mode::GameMode::GridScreen && active_combat.is_none() {
            if let Some(attacker_idx) = units.iter().position(|u| u.state == UnitState::Attacking) {
                let attacker_faction = units[attacker_idx].faction;
                if let Some(defender_idx) = units
                    .iter()
                    .position(|u| u.faction != attacker_faction && u.is_alive())
                {
                    active_combat = Some((attacker_idx, defender_idx));
                }
            }
        }

        let sprite_size = 500.0;
        let overlap = 185.0;
        let center_x = SCREEN_WIDTH as f32 / 2.0;
        let left_x = center_x - sprite_size + overlap;
        let right_x = center_x - overlap;

        // faire disparaître le message de résultat du minigame après un certain temps
        if let Some((_, time_left, _)) = &mut result_display {
            *time_left -= delta_time;
            if *time_left <= 0.0 {
                result_display = None;
            }
        }

        if let Some((attacker_idx, defender_idx)) = active_combat {
            let (attacker_target_x, defender_target_x) = if units[attacker_idx].facing_left {
                (right_x, left_x)
            } else {
                (left_x, right_x)
            };

            let attacker_class = units[attacker_idx].class;
            let defender_class = units[defender_idx].class;
            let (attacker, defender) = two_mut(&mut units, attacker_idx, defender_idx);

            {
                let attacker_set = assets.animation_set_mut(attacker_class);
                combat::start_attack_if_needed(
                    attacker,
                    defender,
                    &mut attacker_set.attack,
                    &mut attacker_set.attack_effect,
                    &mut attack_animation_started,
                    &mut game_mode,
                    &mut attacker_combat_x,
                    &mut defender_combat_x,
                );
            }
            combat::enter_combat(
                attacker,
                defender,
                &mut attacker_combat_x,
                &mut defender_combat_x,
                attacker_target_x,
                defender_target_x,
                &mut combat_entering_timer,
                delta_time,
            );
            combat::start_attack_after_ready(
                attacker,
                defender,
                attack_animation_started,
                &mut combat_ready_timer,
                delta_time,
            );
            {
                let attacker_set = assets.animation_set_mut(attacker_class);
                if let Some(result) = minigame::handle_minigame(
                    attacker,
                    defender,
                    player_faction,
                    &mut timing_bar,
                    &mut damage_multiplier,
                    &rl,
                    delta_time,
                    &mut attacker_set.attack,
                ) {
                    result_display = Some((result, 1.0, damage_multiplier));
                }
            }
            let (attacker_attack, defender_hurt): (&Animation, &mut Animation) =
                match (attacker_class, defender_class) {
                    (UnitClass::Soldier, UnitClass::Wraith) => {
                        (&assets.soldier.attack, &mut assets.wraith.hurt)
                    }
                    (UnitClass::Cavalry, UnitClass::Wraith) => {
                        (&assets.cavalry.attack, &mut assets.wraith.hurt)
                    }
                    (UnitClass::Wraith, UnitClass::Soldier) => {
                        (&assets.wraith.attack, &mut assets.soldier.hurt)
                    }
                    (UnitClass::Wraith, UnitClass::Cavalry) => {
                        (&assets.wraith.attack, &mut assets.cavalry.hurt)
                    }
                    _ => unreachable!("attaquant et défenseur ne sont jamais de la même classe"),
                };
            combat::resolve_attack(
                attacker,
                defender,
                attacker_attack,
                defender_hurt,
                &mut attack_animation_started,
                damage_multiplier,
            );
            {
                let defender_set = assets.animation_set_mut(defender_class);
                combat::update_hurt_state(defender, &defender_set.hurt, &mut defender_set.die);
                combat::update_dying_state(defender, &defender_set.die);
            }
            combat::combat_exit_pause_timer(
                defender,
                attack_animation_started,
                &mut game_mode,
                &mut combat_exit_pause_timer,
                delta_time,
            );
        }

        if game_mode == game_mode::GameMode::GridScreen {
            if units[0].state == UnitState::Dead {
                game_mode = game_mode::GameMode::Defeat;
            } else if units[1].state == UnitState::Dead {
                game_mode = game_mode::GameMode::Victory;
            }
        }

        if current_turn == game_mode::TurnPhase::EnemyTurn
            && enemy_turn_delay <= 0.0
            && game_mode == game_mode::GameMode::GridScreen
            && units[1].state == UnitState::Idle
        {
            current_turn = game_mode::TurnPhase::PlayerTurn;
            for u in units.iter_mut().filter(|u| u.faction == Faction::Human) {
                u.start_turn();
            }
        }

        if active_combat.is_some() && game_mode == game_mode::GameMode::GridScreen {
            active_combat = None;
        }

        let (move_range, came_from) = if let Some(sel) = selected_unit {
            if game_mode == game_mode::GameMode::GridScreen {
                // toute autre unité vivante (alliée ou ennemie) bloque le passage
                let mut occupied = blocked_tiles.clone();
                for (i, u) in units.iter().enumerate() {
                    if i != sel && u.is_alive() {
                        occupied.push((u.grid_x, u.grid_y));
                    }
                }
                MovementRange::compute_movement_range(
                    units[sel].grid_x,
                    units[sel].grid_y,
                    units[sel].move_points_remaining,
                    GRID_COLS,
                    GRID_ROWS,
                    &occupied,
                )
            } else {
                (Vec::new(), HashMap::new())
            }
        } else {
            (Vec::new(), HashMap::new())
        };

        input::cancel_pressed(&rl);

        let cursor_grid_x = mouse_position.x as i32 / TILE_SIZE;
        let cursor_grid_y = mouse_position.y as i32 / TILE_SIZE;

        let clicked = mouse_is_clicked(&rl);

        // sélection / désélection d'une unité Human au clic sur sa case
        // pas d'undead encore
        if game_mode == game_mode::GameMode::GridScreen && clicked && !click_consumed {
            match selected_unit {
                None => {
                    selected_unit = units.iter().position(|u| {
                        u.faction == Faction::Human
                            && u.is_alive()
                            && u.state == UnitState::Idle
                            && u.grid_x == cursor_grid_x
                            && u.grid_y == cursor_grid_y
                    });
                }
                Some(sel)
                    if units[sel].grid_x == cursor_grid_x && units[sel].grid_y == cursor_grid_y =>
                {
                    selected_unit = None;
                }
                _ => {}
            }
        }

        // if pour bouger le personnage sélectionné
        if let Some(sel) = selected_unit {
            if input::handle_movement_normal_click(
                &rl,
                &mut units[sel],
                &move_range,
                &came_from,
                cursor_grid_x,
                cursor_grid_y,
            ) {
                selected_unit = None;
            }
        }

        // une seule attaque par tour et par unité
        let valid_attack_positions = match selected_unit {
            Some(sel) if !units[sel].has_attacked => MovementRange::compute_attackable_positions(
                &move_range,
                units[1].grid_x,
                units[1].grid_y,
                units[sel].attack_range,
                &units[1],
            ),
            _ => Vec::new(),
        };
        let wraith_attackable = !valid_attack_positions.is_empty();

        // if pour attaquer l'ennemi et bouger le personnage sélectionné si il y a qu'une
        // seule case possible pour attaquer
        if let Some(sel) = selected_unit {
            let (mover, enemy) = two_mut(&mut units, sel, 1);
            if input::handle_movement_attack_click(
                &rl,
                mover,
                &came_from,
                &valid_attack_positions,
                wraith_attackable,
                enemy,
                cursor_grid_x,
                cursor_grid_y,
            ) {
                if mover.state != UnitState::ChoosingPosition {
                    selected_unit = None;
                }
            }
        }

        // if pour bouger le personnage vers la case choisie pour attaquer l'ennemi
        if let Some(sel) = selected_unit {
            if input::handle_movement_choosing_position_click(
                &rl,
                &mut units[sel],
                &came_from,
                &valid_attack_positions,
                cursor_grid_x,
                cursor_grid_y,
            ) {
                selected_unit = None;
            }
        }

        if let Some(sel) = selected_unit {
            if input::cancel_pressed(&rl) && units[sel].state == UnitState::ChoosingPosition {
                units[sel].state = UnitState::Idle;
                selected_unit = None;
            }
        }

        let hovering_selectable_unit = selected_unit.is_none()
            && units.iter().any(|u| {
                u.faction == Faction::Human
                    && u.is_alive()
                    && u.state == UnitState::Idle
                    && u.grid_x == cursor_grid_x
                    && u.grid_y == cursor_grid_y
            });

        cursor.update_cursor(
            cursor_grid_x,
            cursor_grid_y,
            hovering_selectable_unit,
            selected_unit.is_some(),
        );

        // drawing --------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        if game_mode == game_mode::GameMode::TitleScreen {
            render::draw_title_screen(&mut d, &assets, &hud, mouse_position);
        } else if game_mode == game_mode::GameMode::FactionSelectionScreen {
            render::draw_faction_selection_screen(
                &mut d,
                &thread,
                &mut assets,
                delta_time,
                mouse_position,
            );
        } else if game_mode == game_mode::GameMode::CombatScreen {
            if let Some((attacker_idx, defender_idx)) = active_combat {
                render::draw_combat_screen(
                    &mut d,
                    &thread,
                    &mut assets,
                    delta_time,
                    &units[attacker_idx],
                    &units[defender_idx],
                    attacker_combat_x,
                    defender_combat_x,
                    timing_bar.as_ref(),
                    result_display.as_ref(),
                );
            }
        } else {
            render::draw_grid_screen(
                &mut d,
                &thread,
                &mut assets,
                delta_time,
                &tile_map,
                &hud,
                &units[0],
                &units[1],
                &units[2],
                game_mode,
                &current_turn,
                wait_button_visible,
                &move_range,
                &valid_attack_positions,
                wraith_attackable,
                cursor_grid_x,
                cursor_grid_y,
                cursor.cursor_type,
                mouse_position,
            );
        }
    }
}
