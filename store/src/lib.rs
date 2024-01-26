use std::{fmt::Formatter, net::{IpAddr, Ipv4Addr, SocketAddr}};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use renetcode::NETCODE_USER_DATA_BYTES;

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

#[derive(Component, Clone, Debug, Default, Serialize, Deserialize)]
pub struct StartGame;

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
    #[default]
    Menu,
    Lobby,
    Terrain,
    Game,
}

pub struct Username(String);

impl Username {
    fn to_netcode_user_data(&self) -> [u8; NETCODE_USER_DATA_BYTES] {
        let mut user_data = [0u8; NETCODE_USER_DATA_BYTES];
        if self.0.len() > NETCODE_USER_DATA_BYTES - 8 {
            panic!("Username is too big");
        }
        user_data[0..8].copy_from_slice(&(self.0.len() as u64).to_le_bytes());
        user_data[8..self.0.len() + 8].copy_from_slice(self.0.as_bytes());

        user_data
    }

    fn from_user_data(user_data: &[u8; NETCODE_USER_DATA_BYTES]) -> Self {
        let mut buffer = [0u8; 8];
        buffer.copy_from_slice(&user_data[0..8]);
        let mut len = u64::from_le_bytes(buffer) as usize;
        len = len.min(NETCODE_USER_DATA_BYTES - 8);
        let username = String::from_utf8(user_data[8..len + 8].to_vec()).unwrap();
        Self(username)
    }
}
