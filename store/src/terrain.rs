use std::error::Error;

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
    pub fn add(&mut self, tile_type: Terrain, player: Player) -> Result<(), Box<dyn Error>> {
        if (tile_type == Terrain::Mountain
            && self.mountain_count[player as usize] >= MAX_MOUNTAIN_COUNT)
            || (tile_type == Terrain::Water && self.water_count[player as usize] >= MAX_WATER_COUNT)
        {
            return Err("Too many mountains or water".into());
        }

        match tile_type {
            Terrain::Mountain => self.mountain_count[player as usize] += 1,
            Terrain::Water => self.water_count[player as usize] += 1,
            _ => (),
        }

        Ok(())
    }

    pub fn remove(&mut self, tile_type: Terrain, player: Player) {
        match tile_type {
            Terrain::Mountain => self.mountain_count[player as usize] -= 1,
            Terrain::Water => self.water_count[player as usize] -= 1,
            _ => (),
        }
    }
}