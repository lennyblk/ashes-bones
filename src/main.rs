use raylib::prelude::*;
mod ai;
mod animation;
mod assets;
mod combat;
mod command;
mod cursor;
mod duel;
mod game;
mod game_mode;
mod grid;
mod input;
mod map;
mod minigame;
mod movement;
mod network;
mod render;
mod screens;
mod turn;
mod ui;
mod unit;

use std::io;
use std::time::Duration;

use command::Command;
use cursor::{CursorType, Cursors};
use game::Game;
use game_mode::GameMode;
use network::{Connection, Event, Message};
use unit::Faction;
use unit::{Unit, UnitState};

const TILE_SIZE: i32 = 48;
const GRID_COLS: i32 = 25;
const GRID_ROWS: i32 = 16;
const SCREEN_WIDTH: i32 = GRID_COLS * TILE_SIZE;
const SCREEN_HEIGHT: i32 = GRID_ROWS * TILE_SIZE;

/// place dans la partie en ligne. L'hôte choisit sa faction et tire le placement
enum Role {
    Host(Faction),
    Guest,
}

/// online pour le dev : `cargo run -- host [human|undead]` ou `cargo run -- join <ip>`.
/// Sans argument : None, partie solo. Bloque jusqu'à la connexion, avant d'ouvrir la fenêtre
fn connect_from_args() -> Option<(Connection, Role)> {
    let args: Vec<String> = std::env::args().collect();
    let (attempt, role) = match args.get(1).map(String::as_str) {
        Some("host") => {
            let faction = match args.get(2) {
                Some(name) => network::parse_faction(name)
                    .unwrap_or_else(|| quit("faction : human ou undead")),
                None => Faction::Human,
            };
            println!(
                "en attente d'un adversaire sur le port {}...",
                network::PORT
            );
            let host = network::Host::new().unwrap_or_else(|e| quit(e));
            (wait_for(|| host.try_accept()), Role::Host(faction))
        }
        Some("join") => {
            let Some(address) = args.get(2) else {
                quit("usage : cargo run -- join <ip>");
            };
            println!("connexion à {address}...");
            let join = network::Join::new(address.clone());
            (wait_for(|| join.try_connect()), Role::Guest)
        }
        _ => return None,
    };
    let connection = attempt.unwrap_or_else(|e| quit(e));
    println!("connecté");
    Some((connection, role))
}

/// l'hôte envoie le placement dès que sa fenêtre est prête : attente très courte
fn wait_for_start(connection: &Connection) -> (Faction, Vec<(i32, i32)>) {
    loop {
        match connection.poll() {
            Some(Event::Message(Message::Start {
                host_faction,
                positions,
            })) => return (host_faction, positions),
            Some(Event::Message(_)) => quit("message inattendu avant le début de partie"),
            Some(Event::Disconnected) => quit("l'hôte s'est déconnecté"),
            None => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

/// redemande à chaque instant jusqu'à avoir une réponse : pas de fenêtre à dessiner ici
fn wait_for(mut attempt: impl FnMut() -> Option<io::Result<Connection>>) -> io::Result<Connection> {
    loop {
        if let Some(result) = attempt() {
            return result;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn quit(error: impl std::fmt::Display) -> ! {
    eprintln!("connexion impossible : {error}");
    std::process::exit(1);
}

fn main() {
    let network = connect_from_args();

    // init window
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Ashes&Bones")
        .build();
    rl.hide_cursor();
    rl.set_exit_key(None); // Échap ouvre le menu pause, ferme plus le jeu

    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    rl.set_random_seed(seed);

    let mut assets = assets::load_assets(&mut rl, &thread);

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

    let hud = ui::HudRects::new();

    let tile_map = map::TileMap::load(&mut rl, &thread, "assets/maps/ashes-bones-map.tmx");

    let mut game = Game::new(&rl, tile_map.blocked_tiles.clone());

    // online : l'hôte a tiré le placement et l'envoie avec sa faction, l'invité l'attend,
    // l'applique et prend l'autre faction. Les deux arrivent direct sur la grille
    if let Some((connection, role)) = &network {
        game.player_faction = match role {
            Role::Host(faction) => {
                connection.send(&Message::Start {
                    host_faction: *faction,
                    positions: game.unit_positions(),
                });
                *faction
            }
            Role::Guest => {
                let (host_faction, positions) = wait_for_start(connection);
                if !game.place_units(&positions) {
                    quit("placement de départ invalide");
                }
                host_faction.opposite()
            }
        };
        game.game_mode = GameMode::GridScreen;
    }

    // run window --------------------------------------------------------------
    while !rl.window_should_close() {
        let delta_time = rl.get_frame_time();
        // temps accéléré pour la grille seulement (bouton x2), le combat garde le vrai temps
        let grid_dt = if game.game_mode == game_mode::GameMode::GridScreen {
            delta_time * game.grid_speed
        } else {
            delta_time
        };

        let mouse_position = rl.get_mouse_position();
        // en pause ou dans le guide, la partie est gelée : ni mouvement ni animation
        if !matches!(
            game.game_mode,
            game_mode::GameMode::PauseScreen | game_mode::GameMode::GuideScreen
        ) {
            for unit in game.units.iter_mut() {
                unit.update_position(grid_dt);
                unit.advance_path();
            }
        }

        let mut click_consumed = false;

        screens::pause::handle_escape(&mut game, &rl);

        // écrans de menu, dans le même ordre qu'avant : un clic peut changer d'écran
        // et l'écran suivant le voit dans la même frame (d'où click_consumed)
        if screens::title::update(&mut game, &rl, &hud, mouse_position, &mut click_consumed) {
            break;
        }
        if screens::pause::update(&mut game, &rl, &hud, mouse_position) {
            break;
        }
        screens::guide::update(&mut game, &rl, &hud, mouse_position);
        screens::faction_selection::update(&mut game, &rl, mouse_position, click_consumed);
        if screens::game_over::update(&mut game, &rl, &hud, mouse_position, &mut click_consumed) {
            break;
        }
        let player_units_busy = game
            .units
            .iter()
            .any(|u| u.faction == game.player_faction && u.is_busy());
        let player_can_act = game.current_turn == game_mode::TurnPhase::PlayerTurn
            && game.game_mode == game_mode::GameMode::GridScreen
            && !player_units_busy;

        // bouton wait
        let wait_button_visible =
            player_can_act && game.selected_unit.is_some_and(|i| game.units[i].can_wait());
        if wait_button_visible
            && input::is_button_clicked(mouse_position, input::mouse_is_clicked(&rl), hud.btn_wait)
        {
            if let Some(sel) = game.selected_unit {
                command::apply(&mut game, Command::Wait { unit: sel });
            }
            game.selected_unit = None;
            click_consumed = true;
        }

        // bouton x1 / x2 (ou touche F), dispo aussi pendant le tour ennemi
        if game.game_mode == game_mode::GameMode::GridScreen
            && (input::speed_toggle_pressed(&rl)
                || input::is_button_clicked(
                    mouse_position,
                    input::mouse_is_clicked(&rl),
                    hud.btn_speed,
                ))
        {
            game.grid_speed = if game.grid_speed > 1.0 { 1.0 } else { 2.0 };
            click_consumed = true;
        }

        // fin de tour : bouton end turn, ou auto quand tous les persos ont fini
        let end_turn_clicked = input::is_button_clicked(
            mouse_position,
            input::mouse_is_clicked(&rl),
            hud.btn_end_turn,
        );

        let enemy_snapshot: Vec<Unit> = game
            .units
            .iter()
            .filter(|u| u.faction == game.player_faction.opposite())
            .cloned()
            .collect();
        let ally_snapshot: Vec<Unit> = game
            .units
            .iter()
            .filter(|u| u.faction == game.player_faction)
            .cloned()
            .collect();
        let all_units_finished = game
            .units
            .iter()
            .filter(|u| u.faction == game.player_faction && u.is_alive())
            .all(|u| u.has_finished_turn(&enemy_snapshot, &ally_snapshot));

        if player_can_act && (end_turn_clicked || all_units_finished) {
            command::apply(&mut game, Command::EndTurn);
            click_consumed = true;
        }

        turn::update_enemy_turn(&mut game, grid_dt);

        combat::update(&mut game, &mut assets, &rl, delta_time);

        if game.game_mode == game_mode::GameMode::GridScreen {
            let player_alive = game
                .units
                .iter()
                .any(|u| u.faction == game.player_faction && u.is_alive());
            let enemy_alive = game
                .units
                .iter()
                .any(|u| u.faction != game.player_faction && u.is_alive());
            if !player_alive {
                game.game_mode = game_mode::GameMode::Defeat;
            } else if !enemy_alive {
                game.game_mode = game_mode::GameMode::Victory;
            }
        }

        turn::end_enemy_turn_if_done(&mut game);

        if game.active_combat.is_some() && game.game_mode == game_mode::GameMode::GridScreen {
            game.active_combat = None;
        }

        // souris sur le bouton x2 : aucune case visée, sinon le clic ferait aussi
        // marcher l'unité sélectionnée vers la case sous le bouton
        let over_speed_button = game.game_mode == game_mode::GameMode::GridScreen
            && hud.btn_speed.check_collision_point_rec(mouse_position);
        let (cursor_grid_x, cursor_grid_y) = if over_speed_button {
            (-1, -1)
        } else {
            (
                mouse_position.x as i32 / TILE_SIZE,
                mouse_position.y as i32 / TILE_SIZE,
            )
        };

        let (highlights, grid_command) =
            grid::update(&mut game, &rl, click_consumed, cursor_grid_x, cursor_grid_y);
        if let Some(cmd) = grid_command {
            command::apply(&mut game, cmd);
        }

        let hovering_selectable_unit = game.current_turn == game_mode::TurnPhase::PlayerTurn
            && game.selected_unit.is_none()
            && game.units.iter().any(|u| {
                u.faction == game.player_faction
                    && u.is_alive()
                    && u.state == UnitState::Idle
                    && u.grid_x == cursor_grid_x
                    && u.grid_y == cursor_grid_y
            });

        cursor.update_cursor(
            cursor_grid_x,
            cursor_grid_y,
            hovering_selectable_unit,
            game.selected_unit.is_some(),
        );

        // drawing --------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        if game.game_mode == game_mode::GameMode::TitleScreen {
            render::draw_title_screen(&mut d, &assets, &hud, mouse_position);
        } else if game.game_mode == game_mode::GameMode::FactionSelectionScreen {
            render::draw_faction_selection_screen(&mut d, &mut assets, delta_time, mouse_position);
        } else if game.game_mode == game_mode::GameMode::PauseScreen {
            render::draw_pause_screen(&mut d, &assets, &hud, mouse_position);
        } else if game.game_mode == game_mode::GameMode::GuideScreen {
            render::draw_guide_screen(
                &mut d,
                &mut assets,
                delta_time,
                &hud,
                mouse_position,
                game.guide_scroll,
            );
        } else if game.game_mode == game_mode::GameMode::CombatScreen {
            if let Some((attacker_idx, defender_idx)) = game.active_combat {
                render::draw_combat_screen(
                    &mut d,
                    &mut assets,
                    delta_time,
                    &game.units[attacker_idx],
                    &game.units[defender_idx],
                    game.attacker_combat_x,
                    game.defender_combat_x,
                    game.minigame.as_ref(),
                    game.result_display.as_ref(),
                    &game.duel_conditions,
                    game.units[attacker_idx].faction == game.player_faction,
                );
            }
        } else {
            render::draw_grid_screen(
                &mut d,
                &mut assets,
                grid_dt,
                &tile_map,
                &hud,
                &game.units,
                game.game_mode,
                &game.current_turn,
                wait_button_visible,
                &highlights.move_range,
                &highlights.choosing_positions,
                &highlights.attackable_enemy_positions,
                &highlights.healable_ally_positions,
                cursor_grid_x,
                cursor_grid_y,
                cursor.cursor_type,
                mouse_position,
                game.selected_unit.map(|i| &game.units[i]),
                game.inspected_enemy.map(|i| &game.units[i]),
                game.grid_speed,
                highlights.duel_preview.as_ref(),
                &highlights.position_scores,
            );
        }
    }
}
