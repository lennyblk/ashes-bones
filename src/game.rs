use raylib::prelude::*;

use crate::duel::DuelConditions;
use crate::game_mode::{GameMode, TurnPhase};
use crate::minigame::{MiniGame, TimingResult};
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

    pub current_turn: TurnPhase,
    pub enemy_turn_delay: f32,
    // ennemis qui restent à jouer ce tour, un par un
    pub ai_turn_queue: Vec<usize>,
    pub ai_acting_unit: Option<usize>,

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
    pub result_display: Option<(TimingResult, f32, f32)>, // alert qui pop au resultat du minigame
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

            current_turn: TurnPhase::PlayerTurn,
            enemy_turn_delay: 0.0,
            ai_turn_queue: Vec::new(),
            ai_acting_unit: None,

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
        self.attack_animation_started = false;
        self.combat_entering_timer = 0.0;
        self.combat_ready_timer = 0.0;
        self.combat_exit_pause_timer = 0.0;
        self.selected_unit = None;
        self.inspected_enemy = None;
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

/// gauche de la rivière (colonnes 11-12), undead à droite, un humain sur la petite
/// île en haut à gauche. Appelée à chaque vrai début de partie (pas juste au lancement)
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
        spawn_unit("Blood Knight",Faction::Undead, UnitClass::BloodKnight, 2, 200, 1, 65, 14, false),
        spawn_unit("Banshee",     Faction::Undead, UnitClass::Banshee,     2,  60, 2, 78,  2, false),
        spawn_unit("Ghoul",       Faction::Undead, UnitClass::Ghoul,       4,  85, 1, 60,  5, false),
        spawn_unit("Skeleton",    Faction::Undead, UnitClass::Skeleton,    2,  80, 1, 50,  6, false),
        spawn_unit("Necromancer", Faction::Undead, UnitClass::Necromancer, 2,  60, 2, 75,  2, true),
    ];

    let river_x = 11;
    let mut taken_tiles: Vec<(i32, i32)> = vec![(2, 2)];
    place_unit(&mut units[0], 2, 2);
    for unit in units.iter_mut().skip(1) {
        let (x_min, x_max) = if unit.faction == Faction::Human {
            (0, river_x - 1)
        } else {
            (river_x + 2, GRID_COLS - 1)
        };
        let tile = random_open_tile(rl, blocked_tiles, &taken_tiles, x_min, x_max);
        place_unit(unit, tile.0, tile.1);
        taken_tiles.push(tile);
    }
    units
}
