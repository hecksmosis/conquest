use itertools::iproduct;
use std::collections::VecDeque;

use bevy::utils::HashSet;

use crate::*;

pub const ADYACENCIES: [(f32, f32); 4] = [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)];

#[derive(Resource, Debug)]
pub struct TileGrid {
    pub grid: [(Option<Player>, bool); (MAP_WIDTH * MAP_HEIGHT * 4.0 + 1.0) as usize],
}

impl TileGrid {
    pub fn in_bounds(x: f32, y: f32) -> bool {
        x >= -MAP_WIDTH && y >= -MAP_HEIGHT && x < MAP_WIDTH && y < MAP_HEIGHT
    }

    pub fn in_bounds_int(x: i32, y: i32) -> bool {
        x >= -MAP_WIDTH as i32
            && y >= -MAP_HEIGHT as i32
            && x < MAP_WIDTH as i32
            && y < MAP_HEIGHT as i32
    }

    pub fn get_index(Vec2 { x, y }: Vec2) -> usize {
        if !Self::in_bounds(x, y) {
            return GRID_SIZE - 1;
        }

        (y + MAP_HEIGHT) as usize * (MAP_WIDTH as usize * 2) + (x + MAP_WIDTH) as usize
    }

    pub fn get_index_from_position(pos: &Position) -> usize {
        Self::get_index(pos.as_grid_index())
    }

    pub fn get_tile(&self, index: Vec2) -> (Option<Player>, bool) {
        self.grid[Self::get_index(index)]
    }

    pub fn get_tile_int(&self, x: i32, y: i32) -> (Option<Player>, bool) {
        self.grid[Self::get_index(Vec2::new(x as f32, y as f32))]
    }

    pub fn set_owner(&mut self, (index, owner): (&Position, &Owned)) {
        let idx = Self::get_index_from_position(index);

        if idx != GRID_SIZE - 1 {
            self.grid[idx].0 = owner.0;
        }
    }

    pub fn make_base(&mut self, owner: Player) {
        self.grid[Self::get_index(match owner {
            Player::Red => Vec2::new(-MAP_WIDTH, -MAP_HEIGHT),
            Player::Blue => Vec2::new(MAP_WIDTH - 1.0, MAP_HEIGHT - 1.0),
        })] = (Some(owner), true)
    }

    pub fn get_connected_tiles(&self, pos: Vec2, owner: Player) -> Vec<Vec2> {
        ADYACENCIES
            .iter()
            .filter_map(|&(dx, dy)| {
                let next_pos = pos + Vec2::new(dx, dy);
                self.get_tile(next_pos)
                    .0
                    .filter(|&p| p == owner)
                    .map(|_| next_pos)
            })
            .collect()
    }

    pub fn get_any_connected(&self, pos: Vec2, owner: Player) -> Option<Vec2> {
        self.get_connected_tiles(pos, owner).first().copied()
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
            if let (Some(p), true) = self.get_tile_int(x, y) {
                if p == player {
                    return true;
                }
            }

            for (&x, &y) in iproduct!([x - 1, x + 1].iter(), [y - 1, y + 1].iter()) {
                if Self::in_bounds_int(x, y) && !visited.contains(&(x, y)) {
                    if let (Some(tile), _) = self.get_tile_int(x, y) {
                        if tile == player {
                            queue.push_back((x, y));
                            visited.insert((x, y));
                        }
                    }
                }
            }
        }

        false
    }
}
