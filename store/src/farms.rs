use crate::*;

#[derive(Resource, Clone, Debug, Default)]
pub struct FarmCounter {
    pub counts: [usize; 2],
    pub points: [usize; 2],
}

impl FarmCounter {
    pub fn available_farms(&self) -> [usize; 2] {
        [self.counts[0] - self.points[0], self.counts[1] - self.points[1]]
    }

    pub fn update(&mut self, grid: &TileGrid) {
        self.counts = [1, 1]; // Base farm
        self.points = [0, 0];

        for tile in grid.get_tiles() {
            if let TileType::Occupied { player_tile: PlayerTile::Farm, owner, level, .. } = tile {
                self.counts[*owner as usize] += level;
            }

            if let TileType::Occupied { player_tile: PlayerTile::Tile, owner, level, .. } = tile {
                self.counts[*owner as usize] += level;
            }
        }
    }
}
