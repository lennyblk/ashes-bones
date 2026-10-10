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

use std::collections::VecDeque;
use std::io;
use std::time::Duration;

use command::Command;
use cursor::{CursorType, Cursors};
use game::Game;
use game_mode::{GameMode, TurnPhase};
use network::{Announcer, Browser, Connection, Event, GameInfo, Message};
use screens::host::HostAction;
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

/// une décision du joueur local : on l'applique, et en ligne on l'envoie à l'adversaire
fn play(game: &mut Game, connection: Option<&Connection>, command: Command) {
    command::apply(game, command);
    if let Some(connection) = connection {
        connection.send(&Message::Command(command));
    }
}

/// hôte : la partie vient d'être tirée (lancement ou revanche), on l'envoie à l'invité
fn send_start(game: &Game, connection: &Connection) {
    connection.send(&Message::Start {
        host_faction: game.player_faction,
        positions: game.unit_positions(),
    });
}

/// invité : applique le placement de l'hôte et prend l'autre faction. L'hôte joue en premier
fn apply_start(game: &mut Game, host_faction: Faction, positions: &[(i32, i32)]) -> bool {
    if !game.place_units(positions) {
        return false;
    }
    game.player_faction = host_faction.opposite();
    game.current_turn = TurnPhase::EnemyTurn;
    true
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
        match role {
            Role::Host(faction) => {
                game.player_faction = *faction;
                send_start(&game, connection);
            }
            Role::Guest => {
                let (host_faction, positions) = wait_for_start(connection);
                if !apply_start(&mut game, host_faction, &positions) {
                    quit("placement de départ invalide");
                }
            }
        }
        game.game_mode = GameMode::GridScreen;
        game.online = true;
    }
    // l'hôte retire le placement à chaque revanche
    let mut is_host = matches!(network, Some((_, Role::Host(_))));
    let mut connection: Option<Connection> = network.map(|(connection, _)| connection);
    // commandes reçues de l'adversaire, pas encore jouées
    let mut incoming: VecDeque<Command> = VecDeque::new();
    // écran Multiplayer : écoute des parties annoncées sur le réseau local
    let mut browser: Option<Browser> = None;
    // hôte en attente d'un adversaire (écran Host) : le port 6666 est ouvert
    let mut host_listener: Option<network::Host> = None;
    // annonce de ma partie sur le réseau local : pendant l'attente ("open") et le match ("full")
    let mut announcer: Option<Announcer> = None;

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
        // l'écran Multiplayer écoute les annonces tant qu'il est affiché, et libère le port 6667
        // en le quittant : un seul programme par PC peut l'écouter
        if game.game_mode == GameMode::MultiplayerScreen {
            if browser.is_none() {
                browser = Browser::new().ok(); // port pris : None, on réessaie à la frame suivante
            }
            if let Some(browser) = &mut browser {
                browser.update();
            }
        } else {
            browser = None;
        }
        screens::multiplayer::update(
            &mut game,
            &rl,
            &hud,
            mouse_position,
            browser.as_mut(),
            click_consumed,
        );
        // après multiplayer : son Back et celui de l'écran Host sont au même endroit, le clic
        // qui revient à la liste ne doit pas aussi la quitter dans la même frame
        let host_action = screens::host::update(
            &mut game,
            &mut rl,
            &hud,
            mouse_position,
            click_consumed,
            host_listener.is_some(),
        );
        match host_action {
            Some(HostAction::Open) => {
                let form = &game.host_form;
                let info = GameInfo {
                    name: form.game_name.trim().to_string(),
                    host_name: form.player_name.trim().to_string(),
                    host_faction: form.faction,
                    full: false,
                };
                match (network::Host::new(), Announcer::new(info)) {
                    (Ok(listener), Ok(new_announcer)) => {
                        host_listener = Some(listener);
                        announcer = Some(new_announcer);
                    }
                    _ => {
                        game.host_form.error =
                            Some("Port 6666 busy: is a game already hosted on this PC?")
                    }
                }
            }
            Some(HostAction::Cancel) => host_listener = None,
            None => {}
        }
        // l'hôte attend : dès que quelqu'un arrive, la partie démarre comme avec `-- host`
        let accepted = host_listener
            .as_ref()
            .and_then(|listener| listener.try_accept());
        match accepted {
            Some(Ok(new_connection)) => {
                game.reset(&rl, GameMode::GridScreen);
                game.player_faction = game.host_form.faction;
                game.online = true;
                is_host = true;
                send_start(&game, &new_connection);
                connection = Some(new_connection);
                host_listener = None;
                if let Some(announcer) = &mut announcer {
                    announcer.info.full = true;
                }
            }
            Some(Err(_)) => {
                host_listener = None;
                game.host_form.error = Some("Connection failed, try again");
            }
            None => {}
        }
        // quitter l'écran Host pendant l'attente (Échap) = annuler
        if game.game_mode != GameMode::HostScreen {
            host_listener = None;
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
                play(&mut game, connection.as_ref(), Command::Wait { unit: sel });
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
            play(&mut game, connection.as_ref(), Command::EndTurn);
            click_consumed = true;
        }

        // online : on range ce que l'adversaire a envoyé depuis la dernière frame
        let mut opponent_left = false;
        if let Some(connection) = &connection {
            while let Some(event) = connection.poll() {
                match event {
                    Event::Message(Message::Command(command)) => incoming.push_back(command),
                    // revanche : seul l'hôte en envoie un, une fois que les deux l'ont demandée
                    Event::Message(Message::Start {
                        host_faction,
                        positions,
                    }) => {
                        if !is_host && game.rematch.mine {
                            game.reset(&rl, GameMode::GridScreen);
                            incoming.clear();
                            if !apply_start(&mut game, host_faction, &positions) {
                                game.reset(&rl, GameMode::TitleScreen);
                                game.title_notice = Some("Invalid data from opponent");
                            }
                        }
                    }
                    Event::Message(Message::Retry) => game.rematch.theirs = true,
                    Event::Message(Message::Duel(result)) => {
                        game.duel_exchange.theirs = Some(result)
                    }
                    Event::Disconnected => {
                        opponent_left = true;
                        break;
                    }
                }
            }
        }
        if opponent_left {
            game.reset(&rl, GameMode::TitleScreen);
            game.title_notice = Some("Your opponent disconnected");
        }
        // revanche : ma demande part une seule fois. Quand les deux l'ont demandée, l'hôte
        // tire une nouvelle partie et l'envoie (l'invité l'applique en recevant START)
        if let Some(connection) = &connection {
            if game.rematch.mine && !game.rematch.sent {
                connection.send(&Message::Retry);
                game.rematch.sent = true;
            }
            if is_host && game.rematch.mine && game.rematch.theirs {
                game.reset(&rl, GameMode::GridScreen);
                incoming.clear();
                send_start(&game, connection);
            }
        }
        // en ligne, revenir à l'écran titre (adversaire parti, Back de la pause ou de fin de
        // partie) = quitter la partie : on ferme la connexion, l'adversaire en est prévenu
        // plus d'attente ni de partie : on ne s'annonce plus, la partie disparaît des listes
        if host_listener.is_none() && connection.is_none() {
            announcer = None;
        }
        if let Some(announcer) = &mut announcer {
            announcer.tick();
        }
        if connection.is_some() && game.game_mode == GameMode::TitleScreen {
            // pas d'écrasement de « Your opponent disconnected » quand c'est lui qui est parti
            if game.title_notice.is_none() {
                game.title_notice = Some("You left the game");
            }
            connection = None;
            incoming.clear();
            game.online = false;
        }
        // ses commandes s'appliquent une par une, pendant son tour, quand plus rien ne bouge
        let board_idle =
            game.game_mode == GameMode::GridScreen && !game.units.iter().any(|u| u.is_busy());
        if board_idle && game.current_turn == TurnPhase::EnemyTurn {
            if let Some(command) = incoming.pop_front() {
                if command::is_legal(&game, command, game.player_faction.opposite()) {
                    command::apply(&mut game, command);
                } else {
                    game.reset(&rl, GameMode::TitleScreen);
                    game.title_notice = Some("Invalid data from opponent");
                }
            }
        }

        turn::update_enemy_turn(&mut game, grid_dt);

        combat::update(&mut game, &mut assets, &rl, delta_time);
        // online : mon résultat de mini-jeu, déposé par le combat, part chez l'adversaire
        if let (Some(connection), Some(result)) = (&connection, game.duel_exchange.to_send.take()) {
            connection.send(&Message::Duel(result));
        }

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
            play(&mut game, connection.as_ref(), cmd);
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
            render::draw_title_screen(&mut d, &assets, &hud, mouse_position, game.title_notice);
        } else if game.game_mode == game_mode::GameMode::FactionSelectionScreen {
            render::draw_faction_selection_screen(&mut d, &mut assets, delta_time, mouse_position);
        } else if game.game_mode == game_mode::GameMode::PauseScreen {
            render::draw_pause_screen(&mut d, &assets, &hud, mouse_position);
        } else if game.game_mode == GameMode::HostScreen {
            render::draw_host_screen(
                &mut d,
                &mut assets,
                delta_time,
                &hud,
                mouse_position,
                &game.host_form,
                host_listener.is_some(),
            );
        } else if game.game_mode == GameMode::MultiplayerScreen {
            render::draw_multiplayer_screen(
                &mut d,
                &mut assets,
                delta_time,
                &hud,
                mouse_position,
                browser.as_ref(),
                game.lobby_selected,
            );
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
                game.rematch.mine,
            );
        }
    }
}
