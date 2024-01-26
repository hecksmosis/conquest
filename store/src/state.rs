use std::{sync::OnceLock, vec};

use crate::*;
use bevy::utils::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameInput {
    Mouse(MouseButton),
    Keyboard(KeyCode),
}

pub enum GameAction {
    Attack(Vec<Vec2>),
    MakeFarm(Vec2),
    Upgrade(Vec2),
    Select(Vec2),
    MakeTerrain(Vec2, Terrain),
    SetTerrainMode(Terrain),
    EndTerrainPlacement,
    Deselect,
}

#[derive(Debug, Default)]
pub struct GameState {
    pub id_to_player: HashMap<u64, Player>,
    pub turn: Player,
    pub grid: TileGrid,
    pub attack_controller: AttackController,
    pub terrain_controller: TerrainCounter,
    pub farm_counter: FarmCounter,
    pub game_phase: GamePhase,
}

#[derive(Debug, Default, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct TileChange {
    pub position: Vec2,
    pub tile: TileType,
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum GamePhase {
    #[default]
    TerrainPlacement,
    Game,
}

impl GameState {
    pub fn set_player_id(&mut self, player_id: u64, player: Player) {
        self.id_to_player.insert(player_id, player);
    }

    pub fn is_player(&self, client_id: u64) -> bool {
        if let Some(p) = self.id_to_player.get(&client_id) {
            return *p == self.turn;
        }
        false
    }

    pub fn get_action(&self, tile_event: &TileEvent) -> Option<GameAction> {
        match &tile_event {
            TileEvent::TileAction {
                client_id,
                position,
                action,
            } if self.game_phase == GamePhase::Game => {
                if !self.is_player(*client_id) || !TileGrid::in_bounds_index(position) {
                    println!("invalid action");
                    return None;
                }

                match (
                    self.grid.get_tile(*position).player_tile(),
                    self.grid.get_tile(*position).owner(),
                    action,
                    self.farm_counter.available_farms()[self.turn as usize],
                ) {
                    (Some(PlayerTile::Tile), Some(p), GameInput::Mouse(MouseButton::Right), _)
                        if p == self.turn =>
                    {
                        Some(GameAction::MakeFarm(*position))
                    }
                    (Some(PlayerTile::Tile), Some(p), GameInput::Mouse(MouseButton::Left), 1..)
                    | (Some(PlayerTile::Farm), Some(p), GameInput::Mouse(MouseButton::Left), _)
                        if p == self.turn =>
                    {
                        Some(GameAction::Upgrade(*position))
                    }
                    (.., GameInput::Mouse(MouseButton::Left), available @ 1..)
                        if self
                            .get_targets(tile_event)
                            .map(|t| t.len() <= available)
                            .unwrap_or(false) =>
                    {
                        Some(GameAction::Attack(
                            self.get_targets(tile_event).unwrap_or_default(),
                        ))
                    }
                    _ => None,
                }
            }

            TileEvent::ToggleSelect {
                client_id,
                position,
            } if self.game_phase == GamePhase::Game => {
                if self.is_player(*client_id) && self.attack_controller.selected.is_some() {
                    Some(GameAction::Deselect)
                } else if self.is_player(*client_id)
                    && (self.grid.get_tile(*position).owner() == Some(self.turn)
                        && self.grid.get_tile(*position).player_tile() == Some(PlayerTile::Tile))
                {
                    Some(GameAction::Select(*position))
                } else {
                    None
                }
            }

            TileEvent::TerrainAction {
                client_id,
                position,
                action,
            } if self.game_phase == GamePhase::TerrainPlacement => {
                if !self.is_player(*client_id) {
                    println!("Not valid");
                    return None;
                }

                match (self.grid.get_tile(*position).owner(), action) {
                    (None, GameInput::Mouse(MouseButton::Left))
                        if TileGrid::in_bounds_index(position)
                            && self.terrain_controller.can_add(self.turn) =>
                    {
                        Some(GameAction::MakeTerrain(
                            *position,
                            self.terrain_controller.placement_mode,
                        ))
                    }
                    (None, GameInput::Mouse(MouseButton::Right))
                        if TileGrid::in_bounds_index(position) =>
                    {
                        Some(GameAction::MakeTerrain(*position, Terrain::None))
                    }
                    (None, GameInput::Keyboard(KeyCode::M)) => {
                        Some(GameAction::SetTerrainMode(Terrain::Mountain))
                    }
                    (None, GameInput::Keyboard(KeyCode::W)) => {
                        Some(GameAction::SetTerrainMode(Terrain::Water))
                    }
                    (None, GameInput::Keyboard(KeyCode::Return)) => {
                        Some(GameAction::EndTerrainPlacement)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Consume a game action into the game state.
    pub fn consume(&mut self, action: &GameAction) -> Vec<ClientEvent> {
        match *action {
            GameAction::Attack(ref targets) => {
                self.attack_controller.deselect();
                vec![
                    ClientEvent::TileChanges(
                        targets
                            .iter()
                            .map(|&t| {
                                if self.grid.get_tile_mut(t).damage(1) == 0
                                    || self.grid.get_tile(t).player_tile() == Some(PlayerTile::Farm)
                                {
                                    self.grid.capture(t, self.turn);
                                }
                                TileChange {
                                    position: t,
                                    tile: self.grid.get_tile(t),
                                }
                            })
                            .collect::<Vec<_>>()
                            .into_iter()
                            .chain(self.grid.update())
                            .collect(),
                    ),
                    ClientEvent::Turn({
                        self.turn = self.turn.other();
                        self.turn
                    }),
                    ClientEvent::Farms({
                        self.farm_counter.update(&self.grid);
                        self.farm_counter.available_farms()
                    }),
                    ClientEvent::Deselect,
                ]
            }

            GameAction::Upgrade(position) => {
                self.grid.upgrade(position);
                self.attack_controller.deselect();

                vec![
                    ClientEvent::TileChanges(vec![TileChange {
                        position,
                        tile: self.grid.get_tile(position),
                    }]),
                    ClientEvent::Turn({
                        self.turn = self.turn.other();
                        self.turn
                    }),
                    ClientEvent::Farms({
                        self.farm_counter.update(&self.grid);
                        self.farm_counter.available_farms()
                    }),
                    ClientEvent::Deselect,
                ]
            }

            GameAction::MakeFarm(position) => {
                self.grid.set_tile(
                    position,
                    TileType::Occupied {
                        player_tile: PlayerTile::Farm,
                        terrain: Terrain::None,
                        owner: self.turn,
                        level: 1,
                        hp: 1,
                    },
                );
                self.attack_controller.deselect();

                vec![
                    ClientEvent::TileChanges(vec![TileChange {
                        position,
                        tile: self.grid.get_tile(position),
                    }]),
                    ClientEvent::Turn({
                        self.turn = self.turn.other();
                        self.turn
                    }),
                    ClientEvent::Farms({
                        self.farm_counter.update(&self.grid);
                        self.farm_counter.available_farms()
                    }),
                    ClientEvent::Deselect,
                ]
            }

            GameAction::Select(position) => {
                self.attack_controller
                    .select(position, self.grid.get_tile(position).level().unwrap_or(1));
                vec![ClientEvent::Select(position)]
            }

            GameAction::Deselect => {
                self.attack_controller.deselect();
                vec![ClientEvent::Deselect]
            }

            GameAction::MakeTerrain(position, terrain) => {
                self.grid.set_tile(position, TileType::Empty(terrain));
                self.terrain_controller.set(terrain, self.turn);
                vec![ClientEvent::TileChanges(vec![TileChange {
                    position,
                    tile: self.grid.get_tile(position),
                }])]
            }

            GameAction::SetTerrainMode(terrain) => {
                self.terrain_controller.placement_mode = terrain;
                vec![ClientEvent::TerrainMode(terrain)]
            }

            GameAction::EndTerrainPlacement => {
                if self.turn == Player::Blue {
                    self.game_phase = GamePhase::Game;
                    return vec![
                        ClientEvent::GamePhase(ClientState::Game),
                        ClientEvent::Turn({
                            self.turn = self.turn.other();
                            self.turn
                        }),
                        ClientEvent::Farms({
                            self.farm_counter.update(&self.grid);
                            self.farm_counter.available_farms()
                        }),
                    ];
                }
                self.turn = self.turn.other();
                vec![ClientEvent::Turn(self.turn)]
            }
        }
    }

    pub fn get_targets(&self, tile_event: &TileEvent) -> Option<Vec<Vec2>> {
        let (position, action) = match tile_event {
            TileEvent::TileAction {
                position, action, ..
            } => (*position, *action),
            _ => return None,
        };

        if let GameInput::Mouse(MouseButton::Left) = action {
            let origin = self
                .attack_controller
                .selected
                .or_else(|| self.grid.get_any_connected(position, self.turn))?;

            let level = self
                .attack_controller
                .selected_level
                .or_else(|| self.grid.get_tile(origin).level())?;

            if !attack_is_valid(origin, position, level) {
                println!("invalid attack");
                return None;
            }

            let direction = (position - origin).normalize();
            println!("direction: {}", direction);

            return Some(match level {
                2 => vec![origin + direction, origin + direction * 2.0],
                3 => vec![
                    origin + direction,
                    origin + direction + Vec2::new(direction.y, -direction.x),
                    origin + direction + Vec2::new(-direction.y, direction.x),
                ],
                _ => vec![position],
            });
        }

        None
    }
}

fn attack_vectors() -> &'static HashMap<usize, Vec<Vec2>> {
    static HASHMAP: OnceLock<HashMap<usize, Vec<Vec2>>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert(
            1,
            vec![
                Vec2::new(1.0, 0.0),
                Vec2::new(-1.0, 0.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, -1.0),
            ],
        );
        m.insert(
            2,
            vec![
                Vec2::new(1.0, 0.0),
                Vec2::new(-1.0, 0.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, -1.0),
                Vec2::new(2.0, 0.0),
                Vec2::new(-2.0, 0.0),
                Vec2::new(0.0, 2.0),
                Vec2::new(0.0, -2.0),
            ],
        );
        m.insert(
            3,
            vec![
                Vec2::new(1.0, 0.0),
                Vec2::new(-1.0, 0.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, -1.0),
            ],
        );
        m
    })
}

fn attack_is_valid(origin: Vec2, target: Vec2, level: usize) -> bool {
    let diff = target - origin;
    info!("{}, level: {}", diff, level);

    attack_vectors()
        .get(&level)
        .map_or(false, |v| v.contains(&diff))
}
