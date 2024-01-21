use bevy::utils::HashSet;
use std::collections::VecDeque;

use crate::*;

pub const ADYACENCIES: [(f32, f32); 4] = [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)];

#[derive(Resource, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TileGrid {
    pub grid: [TileType; (MAP_WIDTH * MAP_HEIGHT * 4.0 + 1.0) as usize],
}

impl Serialize for TileGrid {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut grid = [TileType::EMPTY; GRID_SIZE];

        for (i, tile) in self.grid.iter().enumerate() {
            grid[i] = *tile;
        }

        grid.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TileGrid {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let vec: Vec<TileType> = Vec::deserialize(deserializer)?;

        let mut grid = [TileType::EMPTY; GRID_SIZE];
        for (place, element) in grid.iter_mut().zip(vec.iter()) {
            *place = *element;
        }

        Ok(Self { grid })
    }
}

impl Default for TileGrid {
    fn default() -> Self {
        let mut def = Self {
            grid: [TileType::EMPTY; GRID_SIZE],
        };

        def.make_base(Player::Red);
        def.make_base(Player::Blue);

        def
    }
}

impl TileGrid {
    pub fn get_tiles(&self) -> impl Iterator<Item = &TileType> {
        self.grid.iter().take(self.grid.len() - 1)
    }

    pub fn update(&mut self) -> Vec<TileChange> {
        // Remove tiles disconnected from base
        let mut to_remove = Vec::new();

        for (i, tile) in self.get_tiles().enumerate() {
            if let TileType::Occupied {
                player_tile: PlayerTile::Farm | PlayerTile::Tile,
                owner,
                ..
            } = tile
            {
                if !self.is_connected_to_base(get_vec_from_index(i), *owner) {
                    to_remove.push(i);
                }
            }
        }

        println!(
            "Removing: {:?}",
            to_remove
                .iter()
                .map(|&i| get_vec_from_index(i))
                .collect::<Vec<_>>()
        );

        let mut changes = Vec::new();

        for i in to_remove {
            self.grid[i] = TileType::EMPTY;
            changes.push(TileChange {
                position: get_vec_from_index(i),
                tile: TileType::EMPTY,
            });
        }

        changes
    }

    pub fn check_win(&self) -> [bool; 2] {
        // Count the amount of tiles each player has and if 0 then the other player wins
        let mut counts = [0, 0];

        for tile in self.get_tiles() {
            if let TileType::Occupied { owner, .. } = tile {
                counts[*owner as usize] += 1;
            }
        }

        [counts[0] == 0, counts[1] == 0]
    }

    pub fn in_bounds(x: f32, y: f32) -> bool {
        x >= -MAP_WIDTH && y >= -MAP_HEIGHT && x < MAP_WIDTH && y < MAP_HEIGHT
    }

    pub fn in_bounds_index(index: &Vec2) -> bool {
        Self::in_bounds(index.x, index.y)
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

    pub fn get_tile(&self, index: Vec2) -> TileType {
        self.grid[Self::get_index(index)]
    }

    pub fn get_tile_mut(&mut self, index: Vec2) -> &mut TileType {
        let idx = Self::get_index(index);

        &mut self.grid[idx]
    }

    pub fn empty(&mut self, index: Vec2) {
        self.grid[Self::get_index(index)] = TileType::EMPTY;
    }

    pub fn upgrade(&mut self, index: Vec2) {
        self.grid[Self::get_index(index)].upgrade();
    }

    pub fn get_tile_tup(&self, (x, y): (i32, i32)) -> TileType {
        self.grid[Self::get_index(Vec2::new(x as f32, y as f32))]
    }

    pub fn set_tile(&mut self, index: Vec2, tile: TileType) {
        let idx = Self::get_index(index);

        if idx != GRID_SIZE - 1 {
            self.grid[idx] = tile;
        }
    }

    pub fn capture(&mut self, index: Vec2, player: Player) {
        let idx = Self::get_index(index);

        if idx != GRID_SIZE - 1 {
            if let TileType::Occupied {
                player_tile,
                terrain,
                ..
            } = self.grid[idx]
            {
                self.grid[idx] = TileType::Occupied {
                    player_tile,
                    terrain,
                    owner: player,
                    level: 1,
                    hp: terrain.get_health(),
                };
            } else if let TileType::Empty(terrain) = self.grid[idx] {
                self.grid[idx] = TileType::Occupied {
                    player_tile: PlayerTile::Tile,
                    terrain,
                    owner: player,
                    level: 1,
                    hp: terrain.get_health(),
                };
            }
        }
    }

    pub fn make_base(&mut self, owner: Player) {
        self.grid[Self::get_index(match owner {
            Player::Red => Vec2::new(-MAP_WIDTH, -MAP_HEIGHT),
            Player::Blue => Vec2::new(MAP_WIDTH - 1.0, MAP_HEIGHT - 1.0),
        })] = TileType::Occupied {
            player_tile: PlayerTile::Base,
            terrain: Terrain::None,
            owner,
            level: 1,
            hp: 2,
        };
    }

    pub fn get_connected_tiles(&self, pos: Vec2, owner: Player) -> Vec<Vec2> {
        ADYACENCIES
            .iter()
            .filter_map(|&(dx, dy)| {
                let next_pos = pos + Vec2::new(dx, dy);
                self.get_tile(next_pos)
                    .owner()
                    .filter(|&p| p == owner)
                    .map(|_| next_pos)
            })
            .collect()
    }

    pub fn get_any_connected(&self, pos: Vec2, owner: Player) -> Option<Vec2> {
        self.get_connected_tiles(pos, owner).first().copied()
    }

    pub fn is_connected_to_base(&self, start: Vec2, player: Player) -> bool {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        let start_tuple = (start.x as i32, start.y as i32);
        queue.push_back(start_tuple);
        visited.insert(start_tuple);

        while let Some((x, y)) = queue.pop_front() {
            if self.get_tile_tup((x, y)).is_base(player) {
                return true;
            }

            for &dir in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
                let next_pos = (x + dir.0, y + dir.1);

                if !visited.contains(&next_pos)
                    && self
                        .get_tile_tup(next_pos)
                        .owner()
                        .is_some_and(|p| p == player)
                {
                    queue.push_back(next_pos);
                    visited.insert(next_pos);
                }
            }
        }

        false
    }
}

pub fn get_vec_from_index(index: usize) -> Vec2 {
    Vec2::new(
        (index % (MAP_WIDTH as usize * 2)) as f32 - MAP_WIDTH,
        (index / (MAP_WIDTH as usize * 2)) as f32 - MAP_HEIGHT,
    )
}
