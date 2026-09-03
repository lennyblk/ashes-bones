use crate::enemy::Enemy;
use crate::enemy::EnemyState::Dead;
use std::collections::HashMap;

pub struct MovementRange {
    pub reachable_tiles: Vec<(i32, i32)>,
}

impl MovementRange {
    pub fn compute_movement_range(
        start_x: i32,
        start_y: i32,
        move_points: i32,
        grid_cols: i32,
        grid_rows: i32,
        enemy_x: i32,
        enemy_y: i32,
        enemy: &Enemy,
    ) -> (Vec<(i32, i32)>, HashMap<(i32, i32), (i32, i32)>) {
        let mut visited: Vec<(i32, i32)> = vec![(start_x, start_y)];
        let mut queue: Vec<(i32, i32, i32)> = vec![(start_x, start_y, 0)]; // x, y, coût actuel
        let mut index = 0;
        let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();

        while index < queue.len() {
            let (x, y, cost) = queue[index];
            index += 1;

            if cost >= move_points {
                continue;
            }

            let neighbors = [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)];

            for (nx, ny) in neighbors {
                let in_bounds = nx >= 0 && nx < grid_cols && ny >= 0 && ny < grid_rows;
                let already_visited = visited.contains(&(nx, ny));
                let is_enemy = nx == enemy_x && ny == enemy_y;

                let is_blocked_by_enemy = is_enemy && enemy.state != Dead;

                if in_bounds && !already_visited && !is_blocked_by_enemy {
                    visited.push((nx, ny));
                    queue.push((nx, ny, cost + 1));
                    came_from.insert((nx, ny), (x, y));
                }
            }
        }
        (visited, came_from)
    }

    pub fn compute_attackable_positions(
        move_range: &Vec<(i32, i32)>,
        enemy_x: i32,
        enemy_y: i32,
        attack_range: i32,
        enemy: &Enemy,
    ) -> Vec<(i32, i32)> {
        let mut valid_attack_positions: Vec<(i32, i32)> = Vec::new();

        if enemy.state == Dead {
            return valid_attack_positions;
        }

        for (x, y) in move_range {
            let distance = (enemy_x - x).abs() + (enemy_y - y).abs();
            if distance <= attack_range {
                valid_attack_positions.push((*x, *y));
            }
        }
        valid_attack_positions
    }

    pub fn build_waypoints(
        came_from: &HashMap<(i32, i32), (i32, i32)>,
        start: (i32, i32),
        target: (i32, i32),
    ) -> Vec<(i32, i32)> {
        let mut path = Vec::new();
        let mut waypoints = Vec::new();
        let mut current = target;
        while let Some(&prev) = came_from.get(&current) {
            path.push(current);
            current = prev;
        }
        path.reverse();

        if path.is_empty() {
            return waypoints;
        }

        for i in 0..path.len() {
            let precedente = if i == 0 { start } else { path[i - 1] };
            let current = path[i];

            if i == path.len() - 1 {
                waypoints.push(current);
                break;
            }

            let suivante = path[i + 1];
            let direction_entrante = (current.0 - precedente.0, current.1 - precedente.1);
            let direction_sortante = (suivante.0 - current.0, suivante.1 - current.1);
            if direction_entrante != direction_sortante {
                waypoints.push(current);
            }
        }
        waypoints
    }
}
