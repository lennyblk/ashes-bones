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

                if in_bounds && !already_visited {
                    visited.push((nx, ny));
                    queue.push((nx, ny, cost + 1));
                    came_from.insert((nx, ny), (x, y));
                }
            }
        }
        (visited, came_from)
    }
}
