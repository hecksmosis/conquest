use crate::*;

#[derive(Resource, Clone, Debug)]
pub struct TerrainCounter {
    pub placement_mode: Terrain,
    pub mountain_count: [usize; 2],
    pub water_count: [usize; 2],
}

impl Default for TerrainCounter {
    fn default() -> Self {
        Self {
            placement_mode: Terrain::Mountain,
            mountain_count: [0, 0],
            water_count: [0, 0],
        }
    }
}

impl TerrainCounter {
    pub fn can_add(&self, player: Player) -> bool {
        match self.placement_mode {
            Terrain::Mountain => self.mountain_count[player as usize] < MAX_MOUNTAIN_COUNT,
            Terrain::Water => self.water_count[player as usize] < MAX_WATER_COUNT,
            _ => true,
        }
    }

    pub fn set(&mut self, tile_type: Terrain, player: Player) {
        match tile_type {
            Terrain::Mountain => self.mountain_count[player as usize] += 1,
            Terrain::Water => self.water_count[player as usize] += 1,
            Terrain::None => match self.placement_mode {
                Terrain::Mountain => self.mountain_count[player as usize] = self.mountain_count[player as usize].saturating_sub(1),
                Terrain::Water => self.water_count[player as usize] = self.water_count[player as usize].saturating_sub(1),
                _ => (),
            },
        }
    }
}