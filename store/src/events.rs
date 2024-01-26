use crate::*;

#[derive(Event, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum GameEvent {
    StartGame,
    EndGame,
    PlayerJoined {
        player_id: u64,
    },
    PlayerLeft {
        player_id: u64,
    },
    PlayerTurn {
        player_id: u64,
        tile_event: TileEvent,
    },
}

#[derive(Event, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TileEvent {
    TileAction {
        client_id: u64,
        position: Vec2,
        action: GameInput,
    },
    ToggleSelect {
        client_id: u64,
        position: Vec2,
    },
    TerrainAction {
        client_id: u64,
        position: Vec2,
        action: GameInput,
    },
    None,
}

impl TileEvent {
    pub fn is_none(&self) -> bool {
        matches!(self, TileEvent::None)
    }

    pub fn new_action(client_id: u64, button: &MouseButton, position: Vec2) -> Self {
        TileEvent::TileAction {
            client_id,
            position,
            action: GameInput::Mouse(*button),
        }
    }

    pub fn from_input(
        client_id: u64,
        position: Vec2,
        input: GameInput,
        state: &ClientState,
    ) -> Self {
        match input {
            GameInput::Mouse(button) if state == &ClientState::Terrain => TileEvent::TerrainAction {
                client_id,
                position,
                action: GameInput::Mouse(button),
            },
            GameInput::Mouse(button) => TileEvent::TileAction {
                client_id,
                position,
                action: GameInput::Mouse(button),
            },
            GameInput::Keyboard(KeyCode::Space) => TileEvent::ToggleSelect {
                client_id,
                position,
            },
            GameInput::Keyboard(k) => TileEvent::TerrainAction {
                client_id,
                position,
                action: GameInput::Keyboard(k),
            },
        }
    }
}

#[derive(Event, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GameUpdateEvent {}
