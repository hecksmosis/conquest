use std::{fmt::Formatter, net::{IpAddr, Ipv4Addr, SocketAddr}};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub use consts::*;
pub use events::*;
pub use farms::*;
pub use grid::*;
pub use player::*;
pub use state::*;
pub use tiles::*;
pub use terrain::*;

mod consts;
mod events;
mod farms;
mod grid;
mod player;
mod state;
mod tiles;
mod terrain;

pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9000);

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub enum PlayerTile {
    Farm,
    Tile,
    Base,
}

#[derive(Resource, Debug, Default)]
pub struct AttackController {
    pub selected: Option<Vec2>,
    pub selected_level: Option<usize>,
}

impl AttackController {
    pub fn select(&mut self, position: Vec2, level: usize) {
        self.selected = Some(position);
        self.selected_level = Some(level);
    }

    pub fn deselect(&mut self) {
        self.selected = None;
        self.selected_level = None;
    }
}

#[derive(Component, Clone, Debug, PartialEq, Default)]
pub struct Position(pub Vec2);

impl Position {
    pub fn as_grid_index(&self) -> Vec2 {
        (self.0 - TILE_SIZE / 2.0) / TILE_SIZE
    }
}

impl From<Position> for Transform {
    fn from(pos: Position) -> Self {
        Transform::from_translation(pos.0.extend(0.0))
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct Owned(pub Option<Player>);

#[derive(Event, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ClientEvent {
    Init(Box<TileGrid>),
    TileChanges(Vec<TileChange>),
    Select(Vec2),
    Deselect,
    Turn(Player),
    TerrainMode(Terrain),
    Farms([usize; 2]),
    GamePhase(ClientState),
}

#[derive(Component, Copy, Clone, Default, PartialEq, Debug, Eq, Hash, Deserialize, Serialize)]
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

impl std::fmt::Display for Terrain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Terrain::None => write!(f, "None"),
            Terrain::Water => write!(f, "Water"),
            Terrain::Mountain => write!(f, "Mountain"),
        }
    }
}

#[derive(States, Reflect, Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClientState {
    Menu,
    #[default]
    Terrain,
    Game,
}