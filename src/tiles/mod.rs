use crate::*;
use strum::EnumIter;
pub use tile_bundle::*;
pub use tile_grid::*;

pub mod tile_bundle;
pub mod tile_grid;

#[derive(Component, Clone, Debug, Default)]
pub struct Tile(pub TileType);

#[derive(Component, Clone, Debug, Default)]
pub struct Level(pub usize);

impl Level {
    pub fn up(&mut self) {
        self.0 += 1;
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct Health(pub usize);

impl Health {
    pub fn damage(&mut self) -> usize {
        if self.0 > 0 {
            self.0 -= 1;
        }
        self.0
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct Owned(pub Option<Player>);

#[derive(Component, Copy, Clone, PartialEq, Debug, Eq, Hash)]
pub enum TileType {
    Empty(Terrain),
    Occupied(PlayerTile, Terrain),
}

impl Default for TileType {
    fn default() -> Self {
        TileType::EMPTY
    }
}

impl TileType {
    pub const EMPTY: TileType = TileType::Empty(Terrain::None);
    pub const WATER: TileType = TileType::Empty(Terrain::Water);

    pub fn is_farm(&self) -> bool {
        matches!(self, TileType::Occupied(PlayerTile::Farm, _))
    }

    pub fn is_base(&self) -> bool {
        matches!(self, TileType::Occupied(PlayerTile::Base, _))
    }

    pub fn is_tile(&self) -> bool {
        matches!(self, TileType::Occupied(PlayerTile::Tile, _))
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, TileType::Empty(Terrain::None))
    }

    pub fn terrain(&self) -> Terrain {
        match self {
            TileType::Empty(terrain) => *terrain,
            TileType::Occupied(_, terrain) => *terrain,
        }
    }

    pub fn player_tile(&self) -> Option<PlayerTile> {
        match self {
            TileType::Occupied(player_tile, _) => Some(*player_tile),
            _ => None,
        }
    }

    pub fn set_player_tile(&mut self, player_tile: PlayerTile) {
        match self {
            TileType::Empty(terrain) => *self = TileType::Occupied(player_tile, *terrain),
            TileType::Occupied(_, terrain) => *self = TileType::Occupied(player_tile, *terrain),
        }
    }

    pub fn empty(&mut self) {
        if let TileType::Occupied(_, terrain) = self {
            *self = TileType::Empty(*terrain);
        }
    }
}

#[derive(Component, Copy, Clone, Default, PartialEq, Debug, Eq, Hash)]
pub enum Terrain {
    #[default]
    None,
    Water,
    Mountain,
}

impl Terrain {
    pub fn get_health(&self) -> usize {
        match self {
            Terrain::Water => 0,
            Terrain::None => 1,
            Terrain::Mountain => 2,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
pub enum PlayerTile {
    Farm,
    Tile,
    Base,
}

#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash, EnumIter)]
pub enum OreType {
    Iron,
    Copper,
    Gold,
    Diamond,
}
