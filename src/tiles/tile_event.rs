use crate::*;

#[derive(Event, Debug)]
pub enum TileEvent {
    Upgrade {
        target: Vec2,
        hp: usize,
    },
    Attack {
        targets: Vec<Vec2>,
        player_tile: PlayerTile,
    },
    Select(Vec2),
    Deselect,
}

impl TileEvent {
    pub fn unwrap_attack(&self) -> Option<(&Vec<Vec2>, PlayerTile)> {
        match self {
            TileEvent::Attack {
                ref targets,
                player_tile,
            } => Some((targets, *player_tile)),
            _ => None,
        }
    }

    pub fn unwrap_upgrade(&self) -> Option<(&Vec2, usize)> {
        match self {
            TileEvent::Upgrade { ref target, hp } => Some((target, *hp)),
            _ => None,
        }
    }
}