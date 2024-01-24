use crate::*;

#[derive(Component, Copy, Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub enum TileType {
    Empty(Terrain),
    Occupied {
        player_tile: PlayerTile,
        terrain: Terrain,
        owner: Player,
        level: usize,
        hp: usize,
    },
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
        matches!(
            self,
            TileType::Occupied {
                player_tile: PlayerTile::Farm,
                ..
            }
        )
    }

    pub fn is_tile(&self) -> bool {
        matches!(
            self,
            TileType::Occupied {
                player_tile: PlayerTile::Tile,
                ..
            }
        )
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, TileType::Empty(Terrain::None))
    }

    pub fn terrain(&self) -> Terrain {
        match self {
            TileType::Empty(terrain) => *terrain,
            TileType::Occupied { terrain, .. } => *terrain,
        }
    }

    pub fn player_tile(&self) -> Option<PlayerTile> {
        match self {
            TileType::Occupied { player_tile, .. } => Some(*player_tile),
            _ => None,
        }
    }

    pub fn with_owner(
        &self,
        player_tile: PlayerTile,
        player: Player,
        level: usize,
        hp: usize,
    ) -> Self {
        match self {
            TileType::Empty(terrain) | TileType::Occupied { terrain, .. } => TileType::Occupied {
                player_tile,
                terrain: *terrain,
                owner: player,
                level,
                hp,
            },
        }
    }

    pub fn owner(&self) -> Option<Player> {
        match self {
            TileType::Occupied { owner, .. } => Some(*owner),
            _ => None,
        }
    }

    pub fn level(&self) -> Option<usize> {
        match self {
            TileType::Occupied { level, .. } => Some(*level),
            _ => None,
        }
    }

    pub fn is_base(&self, player: Player) -> bool {
        matches!(self, TileType::Occupied{player_tile: PlayerTile::Base, owner, ..} if *owner == player)
    }

    pub fn empty(&mut self) {
        if let TileType::Occupied { terrain, .. } = self {
            *self = TileType::Empty(*terrain);
        }
    }

    pub fn damage(&mut self, damage: usize) -> usize {
        if let TileType::Occupied { hp, .. } = self {
            *hp = hp.saturating_sub(damage);
            *hp
        } else {
            0
        }
    }

    pub fn upgrade(&mut self) {
        if let TileType::Occupied { player_tile, terrain, owner, level, hp } = self {
            *level += 1;
            *hp += 1;
            *self = TileType::Occupied {
                player_tile: *player_tile,
                terrain: *terrain,
                owner: *owner,
                level: *level,
                hp: *hp,
            };
        }
    }
}
