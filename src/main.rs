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

    let tile_map = map::TileMap::load(&mut rl, &thread, "assets/maps/ashes-bones-map.tmx");
    let blocked_tiles = tile_map.blocked_tiles.clone();

    #[allow(clippy::too_many_arguments)]
    fn spawn_unit(
        name: &str,
        faction: Faction,
        class: UnitClass,
        move_points: i32,
        hp: i32,
        attack_range: i32,
        attack_power: i32,
        defense: i32,
    ) -> Unit {
        Unit {
            name: String::from(name),
            faction,
            class,
            grid_x: 0,
            grid_y: 0,
            screen_x: 0.0,
            screen_y: 0.0,
            move_points,
            move_points_remaining: move_points,
            has_attacked: false,
            hp_points: hp,
            hp_max_points: hp,
            path: Vec::new(),
            state: UnitState::Idle,
            facing_left: false,
            attack_range,
            attack_power,
            defense,
            attack_target: false,
        }
    }

    fn place_unit(unit: &mut Unit, grid_x: i32, grid_y: i32) {
        unit.grid_x = grid_x;
        unit.grid_y = grid_y;
        unit.screen_x = grid_to_screen_x(grid_x);
        unit.screen_y = grid_to_screen_y(grid_y);
    }

    /// pioche une case libre (pas bloquée par la map, pas déjà prise) dans une plage de colonnes
    fn random_open_tile(
        rl: &RaylibHandle,
        blocked_tiles: &[(i32, i32)],
        taken_tiles: &[(i32, i32)],
        x_min: i32,
        x_max: i32,
    ) -> (i32, i32) {
        loop {
            let x = rl.get_random_value::<i32>(x_min..=x_max);
            let y = rl.get_random_value::<i32>(0..=GRID_ROWS - 1);
            if !blocked_tiles.contains(&(x, y)) && !taken_tiles.contains(&(x, y)) {
                return (x, y);
            }
        }
    }

    #[rustfmt::skip]
    let mut units: Vec<Unit> = vec![
        // Human
        spawn_unit("Soldier",     Faction::Human,  UnitClass::Soldier,     2, 120, 1, 45, 10),
        spawn_unit("Cavalry",     Faction::Human,  UnitClass::Cavalry,     4,  90, 1, 55,  7),
        spawn_unit("Assassin",    Faction::Human,  UnitClass::Assassin,    4,  65, 1, 85,  2),
        spawn_unit("Longbowman",  Faction::Human,  UnitClass::Longbowman,  2,  75, 3, 50,  5),
        spawn_unit("Mage",        Faction::Human,  UnitClass::Mage,        2,  55, 2, 80,  1),
        spawn_unit("Priest",      Faction::Human,  UnitClass::Priest,      2,  95, 2, 35,  9),

        // Undead
        spawn_unit("Wraith",      Faction::Undead, UnitClass::Wraith,      3,  90, 1, 50,  8),
        spawn_unit("Blood Knight",Faction::Undead, UnitClass::BloodKnight, 2, 200, 1, 65, 14),
        spawn_unit("Banshee",     Faction::Undead, UnitClass::Banshee,     2,  60, 2, 78,  2),
        spawn_unit("Ghoul",       Faction::Undead, UnitClass::Ghoul,       4,  85, 1, 60,  5),
        spawn_unit("Skeleton",    Faction::Undead, UnitClass::Skeleton,    2,  80, 1, 50,  6),
        spawn_unit("Necromancer", Faction::Undead, UnitClass::Necromancer, 2,  60, 2, 75,  2),
    ];

    // dispersion : humains à gauche de la rivière (colonnes 11-12), undead à droite,
    // un humain sur la petite île en haut à gauche
    let river_x = 11;
    let mut taken_tiles: Vec<(i32, i32)> = vec![(2, 2)];
    place_unit(&mut units[0], 2, 2);
    for unit in units.iter_mut().skip(1) {
        let (x_min, x_max) = if unit.faction == Faction::Human {
            (0, river_x - 1)
        } else {
            (river_x + 2, GRID_COLS - 1)
        };
        let tile = random_open_tile(&rl, &blocked_tiles, &taken_tiles, x_min, x_max);
        place_unit(unit, tile.0, tile.1);
        taken_tiles.push(tile);
    }

    // état de départ, pour le bouton retry
    let units_initial = units.clone();

    let mut game_mode = game_mode::GameMode::TitleScreen;
    let mut attacker_combat_x: f32 = 0.0;
    let mut defender_combat_x: f32 = 0.0;
    let mut combat_entering_timer: f32 = 0.0;
    let mut combat_ready_timer: f32 = 0.0;
    let mut combat_exit_pause_timer: f32 = 0.0;

    let mut attack_animation_started = false;
    // (indice attaquant, indice défenseur) dans `units`
    let mut active_combat: Option<(usize, usize)> = None;

    let mut current_turn = game_mode::TurnPhase::PlayerTurn;
    let mut enemy_turn_delay: f32 = 0.0;
    // ennemis qui restent à jouer ce tour, un par un
    let mut ai_turn_queue: Vec<usize> = Vec::new();
    let mut ai_acting_unit: Option<usize> = None;

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
        for unit in units.iter_mut() {
            unit.update_position(delta_time);
            unit.advance_path();
        }

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
        let player_units_busy = units
            .iter()
            .any(|u| u.faction == player_faction && u.is_busy());
        let player_can_act = current_turn == game_mode::TurnPhase::PlayerTurn
            && game_mode == game_mode::GameMode::GridScreen
            && !player_units_busy;

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

        let enemy_snapshot: Vec<Unit> = units
            .iter()
            .filter(|u| u.faction == player_faction.opposite())
            .cloned()
            .collect();
        let all_units_finished = units
            .iter()
            .filter(|u| u.faction == player_faction && u.is_alive())
            .all(|u| u.has_finished_turn(&enemy_snapshot));

        if player_can_act && (end_turn_clicked || all_units_finished) {
            current_turn = game_mode::TurnPhase::EnemyTurn;
            enemy_turn_delay = ENEMY_TURN_DELAY;
            ai_turn_queue = units
                .iter()
                .enumerate()
                .filter(|(_, u)| u.faction == player_faction.opposite() && u.is_alive())
                .map(|(i, _)| i)
                .collect();
            ai_acting_unit = None;
            selected_unit = None;
            click_consumed = true;
        }

        // une unité ennemie à la fois, on attend qu'elle finisse avant la suivante
        if current_turn == game_mode::TurnPhase::EnemyTurn
            && game_mode == game_mode::GameMode::GridScreen
        {
            if let Some(idx) = ai_acting_unit {
                if !units[idx].is_busy() {
                    ai_acting_unit = None;
                    enemy_turn_delay = ENEMY_TURN_DELAY;
                }
            } else if enemy_turn_delay > 0.0 {
                enemy_turn_delay -= delta_time;
            } else if let Some(idx) = ai_turn_queue.pop() {
                if units[idx].is_alive() {
                    let targets: Vec<Unit> = units
                        .iter()
                        .filter(|u| u.faction == player_faction && u.is_alive())
                        .cloned()
                        .collect();
                    // toute autre unité vivante (alliée ou ennemie) bloque le passage
                    let obstacles: Vec<Unit> = units
                        .iter()
                        .enumerate()
                        .filter(|(i, u)| *i != idx && u.is_alive())
                        .map(|(_, u)| u.clone())
                        .collect();
                    ai::take_turn(&mut units[idx], &targets, &obstacles, &blocked_tiles);
                    ai_acting_unit = Some(idx);
                }
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
            let attacker_attack_finished = assets.animation_set_mut(attacker_class).attack.finished;
            let defender_hurt = &mut assets.animation_set_mut(defender_class).hurt;
            combat::resolve_attack(
                attacker,
                defender,
                attacker_attack_finished,
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
            let player_alive = units
                .iter()
                .any(|u| u.faction == player_faction && u.is_alive());
            let enemy_alive = units
                .iter()
                .any(|u| u.faction != player_faction && u.is_alive());
            if !player_alive {
                game_mode = game_mode::GameMode::Defeat;
            } else if !enemy_alive {
                game_mode = game_mode::GameMode::Victory;
            }
        }

        let enemy_turn_queue_done = ai_turn_queue.is_empty() && ai_acting_unit.is_none();
        if current_turn == game_mode::TurnPhase::EnemyTurn
            && enemy_turn_delay <= 0.0
            && game_mode == game_mode::GameMode::GridScreen
            && enemy_turn_queue_done
        {
            current_turn = game_mode::TurnPhase::PlayerTurn;
            for u in units.iter_mut().filter(|u| u.faction == player_faction) {
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

        // sélection / désélection d'une unité du joueur au clic sur sa case
        if game_mode == game_mode::GameMode::GridScreen && clicked && !click_consumed {
            match selected_unit {
                None => {
                    selected_unit = units.iter().position(|u| {
                        u.faction == player_faction
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

        let (valid_attack_positions, attackable_enemy_positions): (
            Vec<(i32, i32)>,
            Vec<(i32, i32)>,
        ) = match selected_unit {
            Some(sel) if !units[sel].has_attacked => {
                let attacker_faction = units[sel].faction;
                let attacker_range = units[sel].attack_range;
                let mut positions = Vec::new();
                let mut enemy_tiles = Vec::new();
                for enemy in units
                    .iter()
                    .filter(|u| u.faction != attacker_faction && u.is_alive())
                {
                    let reachable = MovementRange::compute_attackable_positions(
                        &move_range,
                        enemy.grid_x,
                        enemy.grid_y,
                        attacker_range,
                        enemy,
                    );
                    if !reachable.is_empty() {
                        enemy_tiles.push((enemy.grid_x, enemy.grid_y));
                        for pos in reachable {
                            if !positions.contains(&pos) {
                                positions.push(pos);
                            }
                        }
                    }
                }
                (positions, enemy_tiles)
            }
            _ => (Vec::new(), Vec::new()),
        };

        // if pour attaquer l'ennemi et bouger le personnage sélectionné si il y a qu'une
        // seule case possible pour attaquer
        if let Some(sel) = selected_unit {
            let attacker_faction = units[sel].faction;
            if let Some(enemy_idx) = units.iter().position(|u| {
                u.faction != attacker_faction
                    && u.is_alive()
                    && u.grid_x == cursor_grid_x
                    && u.grid_y == cursor_grid_y
            }) {
                let (mover, enemy) = two_mut(&mut units, sel, enemy_idx);
                if input::handle_movement_attack_click(
                    &rl,
                    mover,
                    &came_from,
                    &valid_attack_positions,
                    !valid_attack_positions.is_empty(),
                    enemy,
                    cursor_grid_x,
                    cursor_grid_y,
                ) {
                    if mover.state != UnitState::ChoosingPosition {
                        selected_unit = None;
                    }
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
                u.faction == player_faction
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
            render::draw_faction_selection_screen(&mut d, &mut assets, delta_time, mouse_position);
        } else if game_mode == game_mode::GameMode::CombatScreen {
            if let Some((attacker_idx, defender_idx)) = active_combat {
                render::draw_combat_screen(
                    &mut d,
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
                &mut assets,
                delta_time,
                &tile_map,
                &hud,
                &units,
                game_mode,
                &current_turn,
                wait_button_visible,
                &move_range,
                &valid_attack_positions,
                &attackable_enemy_positions,
                cursor_grid_x,
                cursor_grid_y,
                cursor.cursor_type,
                mouse_position,
            );
        }
    }
}
