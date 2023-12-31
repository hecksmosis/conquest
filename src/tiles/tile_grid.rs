use std::collections::VecDeque;

use bevy::utils::HashSet;

use crate::*;

#[derive(Resource, Debug)]
pub struct TileGrid {
    pub grid: [(Option<Player>, bool); (MAP_WIDTH * MAP_HEIGHT * 4.0 + 1.0) as usize],
}

impl TileGrid {
    fn in_bounds(x: f32, y: f32) -> bool {
        x >= -MAP_WIDTH && y >= -MAP_HEIGHT && x < MAP_WIDTH && y < MAP_HEIGHT
    }

    fn get_index(Vec2 { x, y }: Vec2) -> usize {
        if !Self::in_bounds(x, y) {
            return GRID_SIZE - 1;
        }

        (y + MAP_HEIGHT) as usize * (MAP_WIDTH as usize * 2) + (x + MAP_WIDTH) as usize
    }

    pub fn get_tile(&self, index: Vec2) -> (Option<Player>, bool) {
        self.grid[Self::get_index(index)]
    }

    pub fn set_owner(&mut self, index: Vec2, owner: Option<Player>) {
        if Self::get_index(index) != GRID_SIZE - 1 {
            self.grid[Self::get_index(index)].0 = owner;
        }
    }

    pub fn make_base(&mut self, owner: Player) {
        self.grid[Self::get_index(match owner {
            Player::Red => Vec2::new(-MAP_WIDTH, -MAP_HEIGHT),
            Player::Blue => Vec2::new(MAP_WIDTH - 1.0, MAP_HEIGHT - 1.0),
        })] = (Some(owner), true)
    }

    pub fn get_connected_tiles(
        &self,
        Vec2 { x, y }: Vec2,
        owner: Player,
    ) -> Vec<(Player, Vec2, bool)> {
        let mut tiles = Vec::new();

        for vec in &[
            Vec2 { x: x + 1.0, y },
            Vec2 { x: x - 1.0, y },
            Vec2 { x, y: y + 1.0 },
            Vec2 { x, y: y - 1.0 },
        ] {
            if let (Some(p), base) = self.get_tile(*vec) {
                if p == owner {
                    tiles.push((p, *vec, base));
                }
            }
        }

        tiles
    }

    pub fn any_adyacent_tiles(&self, position: Vec2, player: Player) -> bool {
        if !Self::in_bounds(position.x, position.y) {
            return false;
        }

        self.get_connected_tiles(position, player).len() != 0
    }

    pub fn is_connected_to_base(&self, start: Vec2, check_player: Player) -> bool {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        let player = self.get_tile(start).0.unwrap_or(check_player);
        if player != check_player {
            return false;
        }

        let start_tuple = (start.x as i32, start.y as i32);
        queue.push_back(start_tuple);
        visited.insert(start_tuple);

        while let Some((x, y)) = queue.pop_front() {
            if let (Some(p), true) = self.get_tile(Vec2 {
                x: x as f32,
                y: y as f32,
            }) {
                if p == player {
                    return true;
                }
            }

            for &dir in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
                let next_pos = (x + dir.0, y + dir.1);

                if Self::in_bounds(next_pos.0 as f32, next_pos.1 as f32)
                    && !visited.contains(&next_pos)
                {
                    if let (Some(tile), _) = self.get_tile(Vec2 {
                        x: next_pos.0 as f32,
                        y: next_pos.1 as f32,
                    }) {
                        if tile == player {
                            queue.push_back(next_pos);
                            visited.insert(next_pos);
                        }
                    }
                }
            }
        }

        false
    }
}
