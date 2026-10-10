use std::net::IpAddr;

use raylib::prelude::*;

use crate::duel::DuelConditions;
use crate::game_mode::{GameMode, TurnPhase};
use crate::minigame::{DuelExchange, MiniGame, ResultDisplay};
use crate::screens::host::HostForm;
use crate::unit::{Faction, Unit, UnitClass, UnitState};
use crate::{GRID_COLS, GRID_ROWS, TILE_SIZE};

pub struct Game {
    pub units: Vec<Unit>,
    pub blocked_tiles: Vec<(i32, i32)>,
    pub player_faction: Faction,
    // mon index dans units (aucune -> None)
    pub selected_unit: Option<usize>,
    // ennemi cliqué dont on affiche les stats (index dans units)
    pub inspected_enemy: Option<usize>,

    pub game_mode: GameMode,
    // écran auquel revenir en quittant le menu pause / le guide
    pub paused_from: GameMode,
    pub guide_return_to: GameMode,
    // scroll du guide en pixels, et drag de sa scrollbar en cours
    pub guide_scroll: f32,
    pub guide_dragging: bool,
    // écran Multiplayer : IP de la partie sélectionnée dans la liste
    pub lobby_selected: Option<IpAddr>,
    // écran Host game : gardé d'une partie à l'autre (pas remis à zéro par reset)
    pub host_form: HostForm,

    pub current_turn: TurnPhase,
    // vitesse de la grille (x1 / x2) : marche, animations, enchaînement des ennemis.
    // Gardée d'une partie à l'autre, n'affecte ni le combat ni les mini-jeux
    pub grid_speed: f32,
    pub enemy_turn_delay: f32,
    // ennemis qui restent à jouer ce tour, un par un
    pub ai_turn_queue: Vec<usize>,
    pub ai_acting_unit: Option<usize>,
    // adversaire en ligne : pas d'IA, ses commandes arrivent par le réseau
    pub online: bool,

    // (indice attaquant, indice défenseur) dans `units`
    pub active_combat: Option<(usize, usize)>,
    pub attacker_combat_x: f32,
    pub defender_combat_x: f32,
    pub combat_entering_timer: f32,
    pub combat_ready_timer: f32,
    pub combat_exit_pause_timer: f32,
    pub attack_animation_started: bool,

    // conditions du duel en cours, figées au lancement du combat (placement sur la grille)
    pub duel_conditions: DuelConditions,
    pub minigame: Option<MiniGame>,
    pub damage_multiplier: f32,
    pub result_display: Option<ResultDisplay>, // alert qui pop au resultat du minigame
    // online : résultats du duel en cours échangés avec l'adversaire
    pub duel_exchange: DuelExchange,

    // online : revanche demandée en fin de partie
    pub rematch: Rematch,

    // message affiché sur l'écran titre (ex : adversaire déconnecté), effacé au prochain clic
    pub title_notice: Option<&'static str>,
}

/// online : revanche en fin de partie. Elle démarre quand les deux ont cliqué Retry
#[derive(Default)]
pub struct Rematch {
    /// j'ai cliqué Retry
    pub mine: bool,
    /// ma demande est partie chez l'adversaire
    pub sent: bool,
    /// l'adversaire a cliqué Retry
    pub theirs: bool,
}

impl Game {
    pub fn new(rl: &RaylibHandle, blocked_tiles: Vec<(i32, i32)>) -> Self {
        Game {
            units: fresh_units(rl, &blocked_tiles),
            blocked_tiles,
            player_faction: Faction::Human,
            selected_unit: None,
            inspected_enemy: None,

            game_mode: GameMode::TitleScreen,
            paused_from: GameMode::GridScreen,
            guide_return_to: GameMode::TitleScreen,
            guide_scroll: 0.0,
            guide_dragging: false,
            lobby_selected: None,
            host_form: HostForm::new(),

            current_turn: TurnPhase::PlayerTurn,
            grid_speed: 1.0,
            enemy_turn_delay: 0.0,
            ai_turn_queue: Vec::new(),
            ai_acting_unit: None,
            online: false,

            active_combat: None,
            attacker_combat_x: 0.0,
            defender_combat_x: 0.0,
            combat_entering_timer: 0.0,
            combat_ready_timer: 0.0,
            combat_exit_pause_timer: 0.0,
            attack_animation_started: false,

            duel_conditions: DuelConditions::neutral(),
            minigame: None,
            damage_multiplier: 1.0,
            result_display: None,
            duel_exchange: DuelExchange::default(),

            title_notice: None,
            rematch: Rematch::default(),
        }
    }

    /// nouvelle partie (retry / back) : nouvelles unités, tour et combat remis à zéro
    pub fn reset(&mut self, rl: &RaylibHandle, next_mode: GameMode) {
        self.units = fresh_units(rl, &self.blocked_tiles);
        self.game_mode = next_mode;
        self.current_turn = TurnPhase::PlayerTurn;
        self.enemy_turn_delay = 0.0;
        self.active_combat = None;
        self.duel_conditions = DuelConditions::neutral();
        self.minigame = None;
        self.duel_exchange = DuelExchange::default();
        self.rematch = Rematch::default();
        self.attack_animation_started = false;
        self.combat_entering_timer = 0.0;
        self.combat_ready_timer = 0.0;
        self.combat_exit_pause_timer = 0.0;
        self.selected_unit = None;
        self.inspected_enemy = None;
    }
}

impl Game {
    /// case de chaque unité, dans l'ordre de `units` : ce que l'hôte envoie à l'invité
    pub fn unit_positions(&self) -> Vec<(i32, i32)> {
        self.units.iter().map(|u| (u.grid_x, u.grid_y)).collect()
    }

    /// place les unités comme chez l'hôte. false si la liste reçue ne colle pas
    pub fn place_units(&mut self, positions: &[(i32, i32)]) -> bool {
        let valid = positions.len() == self.units.len()
            && positions
                .iter()
                .all(|&(x, y)| (0..GRID_COLS).contains(&x) && (0..GRID_ROWS).contains(&y));
        if valid {
            for (unit, &(x, y)) in self.units.iter_mut().zip(positions) {
                place_unit(unit, x, y);
            }
        }
        valid
    }
}

fn grid_to_screen_x(grid_x: i32) -> f32 {
    let x = (grid_x * TILE_SIZE) as f32 + (-40.0);
    x
}

fn grid_to_screen_y(grid_y: i32) -> f32 {
    let y = (grid_y * TILE_SIZE) as f32 + (-40.0);
    y
}

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
    can_heal: bool,
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
        can_heal,
        pending_action: None,
    }
}

fn place_unit(unit: &mut Unit, grid_x: i32, grid_y: i32) {
    unit.grid_x = grid_x;
    unit.grid_y = grid_y;
    unit.screen_x = grid_to_screen_x(grid_x);
    unit.screen_y = grid_to_screen_y(grid_y);
}

/// cases libres où un camp peut apparaître : la plus grande zone de terre d'un seul
/// tenant dans ses colonnes. Écarte les îlots coupés par l'eau où une unité serait coincée
fn spawn_area(blocked_tiles: &[(i32, i32)], x_min: i32, x_max: i32) -> Vec<(i32, i32)> {
    let is_open = |x: i32, y: i32| {
        x >= x_min && x <= x_max && y >= 0 && y < GRID_ROWS && !blocked_tiles.contains(&(x, y))
    };
    let mut seen: Vec<(i32, i32)> = Vec::new();
    let mut biggest: Vec<(i32, i32)> = Vec::new();
    for start_x in x_min..=x_max {
        for start_y in 0..GRID_ROWS {
            if !is_open(start_x, start_y) || seen.contains(&(start_x, start_y)) {
                continue;
            }
            // remplissage (flood fill) de la zone qui contient cette case
            let mut zone = vec![(start_x, start_y)];
            seen.push((start_x, start_y));
            let mut i = 0;
            while i < zone.len() {
                let (x, y) = zone[i];
                for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
                    if is_open(nx, ny) && !seen.contains(&(nx, ny)) {
                        seen.push((nx, ny));
                        zone.push((nx, ny));
                    }
                }
                i += 1;
            }
            if zone.len() > biggest.len() {
                biggest = zone;
            }
        }
    }
    biggest
}

/// humains à gauche de la rivière (colonnes 11-12), undead à droite, placés au hasard.
/// Appelée à chaque vrai début de partie (pas juste au lancement)
#[rustfmt::skip]
fn fresh_units(rl: &RaylibHandle, blocked_tiles: &[(i32, i32)]) -> Vec<Unit> {
    let mut units: Vec<Unit> = vec![
        // Human
        spawn_unit("Soldier",     Faction::Human,  UnitClass::Soldier,     2, 120, 1, 45, 10, false),
        spawn_unit("Cavalry",     Faction::Human,  UnitClass::Cavalry,     4,  90, 1, 55,  7, false),
        spawn_unit("Assassin",    Faction::Human,  UnitClass::Assassin,    4,  65, 1, 85,  2, false),
        spawn_unit("Longbowman",  Faction::Human,  UnitClass::Longbowman,  2,  75, 3, 50,  5, false),
        spawn_unit("Mage",        Faction::Human,  UnitClass::Mage,        2,  55, 2, 80,  1, false),
        spawn_unit("Priest",      Faction::Human,  UnitClass::Priest,      2,  95, 2, 35,  9, true),

        // Undead
        spawn_unit("Wraith",      Faction::Undead, UnitClass::Wraith,      3,  90, 1, 50,  8, false),
        spawn_unit("Blood Knight",Faction::Undead, UnitClass::BloodKnight, 4, 120, 1, 65, 14, false),
        spawn_unit("Banshee",     Faction::Undead, UnitClass::Banshee,     2,  60, 2, 78,  2, false),
        spawn_unit("Ghoul",       Faction::Undead, UnitClass::Ghoul,       3,  85, 1, 60,  5, false),
        spawn_unit("Skeleton",    Faction::Undead, UnitClass::Skeleton,    2,  80, 1, 50,  6, false),
        spawn_unit("Necromancer", Faction::Undead, UnitClass::Necromancer, 2,  60, 2, 75,  2, true),
    ];

    let river_x = 11;
    let mut human_tiles = spawn_area(blocked_tiles, 0, river_x - 1);
    let mut undead_tiles = spawn_area(blocked_tiles, river_x + 2, GRID_COLS - 1);
    for unit in units.iter_mut() {
        let free_tiles = if unit.faction == Faction::Human {
            &mut human_tiles
        } else {
            &mut undead_tiles
        };
        // on retire la case tirée : deux unités ne peuvent pas tomber au même endroit
        let pick = rl.get_random_value::<i32>(0..=free_tiles.len() as i32 - 1) as usize;
        let tile = free_tiles.swap_remove(pick);
        place_unit(unit, tile.0, tile.1);
    }
    units
}
