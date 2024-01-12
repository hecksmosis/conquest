#![allow(clippy::type_complexity)]

use std::error::Error;

use assets::{AssetsPlugin, TileAssets};
pub use bevy::prelude::*;
use bevy::{
    utils::HashMap,
    window::{close_on_esc, PrimaryWindow},
};
use camera::CameraPlugin;
use consts::*;
use debug::DebugPlugin;
use farms::{FarmCounter, FarmPlugin};
use grid_mouse::*;
use hud::HUDPlugin;
use menu::{MenuPlugin, WinCounter};
use player::*;
use strum::IntoEnumIterator;
use terrain::TerrainPlugin;
use tiles::*;
use turn::*;
use utils::{
    get_attack_targets, get_rectified_mouse_position, get_selected_grid_position,
    get_vec_from_index,
};

mod assets;
mod camera;
mod consts;
mod debug;
mod farms;
mod grid_mouse;
mod hud;
mod menu;
mod player;
mod terrain;
mod tiles;
mod turn;
mod utils;

#[derive(Component, Clone, Debug, PartialEq, Default)]
struct Position(pub(crate) Vec2);

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

#[derive(Resource, Default)]
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

#[derive(States, Reflect, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Menu,
    Terrain,
    Game,
}

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_plugins((
            DefaultPlugins,
            CameraPlugin,
            GridMousePlugin,
            TurnPlugin,
            HUDPlugin,
            MenuPlugin,
            DebugPlugin,
            AssetsPlugin,
            TerrainPlugin,
            FarmPlugin,
        ))
        .add_event::<GridUpdateEvent>()
        .add_event::<TileEvent>()
        .add_event::<GameOverEvent>()
        .init_resource::<TileGrid>()
        .init_resource::<AttackController>()
        .add_systems(OnEnter(GameState::Terrain), setup.after(menu::cleanup))
        .add_systems(OnExit(GameState::Game), cleanup)
        .add_systems(
            Update,
            (
                (
                    select_tile,
                    tile_attack.pipe(send_events),
                    upgrade.pipe(send_events),
                    register_event.map(noop),
                    update_selection,
                    update_grid
                        .pipe(delete_if_disconnected)
                        .pipe(check_win)
                        .run_if(grid_update_event),
                    main_menu,
                )
                    .run_if(in_state(GameState::Game)),
                close_on_esc,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, mut grid: ResMut<TileGrid>, assets: Res<TileAssets>, mut attack_controller: ResMut<AttackController>) {
    *grid = default();
    (1..(MAP_HEIGHT * MAP_WIDTH * 4.0 - 1.0) as usize).for_each(|index| {
        _ = commands.spawn(TileBundle::blank(get_vec_from_index(index), &assets))
    });

    // Spawn selector
    commands.spawn((
        Selector,
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::splat(TILE_SIZE)),
                ..default()
            },
            texture: assets.selector_texture.clone(),
            visibility: Visibility::Hidden,
            ..default()
        },
    ));

    Player::iter().for_each(|p| {
        commands.spawn(TileBundle::base(p, &assets));
        grid.make_base(p);
    });

    *attack_controller = default();
}

fn cleanup(mut commands: Commands, mut entities: Query<Entity, Or<(With<Tile>, With<Selector>)>>) {
    for e in entities.iter_mut() {
        commands.entity(e).despawn_recursive();
    }
}

fn noop<T>(_: T) {}

fn grid_update_event(mut grid_update: EventReader<GridUpdateEvent>) -> bool {
    grid_update.read().count() == 0
}

fn send_events(
    In(send): In<Option<()>>,
    mut turn_event: EventWriter<TurnEvent>,
    mut grid_update: EventWriter<GridUpdateEvent>,
) {
    if send.is_some() {
        grid_update.send(GridUpdateEvent);
        turn_event.send(TurnEvent);
    }
}

#[derive(Event)]
pub(crate) struct TurnEvent;

#[derive(Event)]
pub struct GridUpdateEvent;

#[derive(Event, Debug)]
pub enum TileEvent {
    UpgradeEvent {
        target: Vec2,
        hp: usize,
    },
    AttackEvent {
        targets: Vec<Vec2>,
        player_tile: PlayerTile,
    },
    SelectEvent(Vec2),
    DeselectEvent,
}

impl TileEvent {
    pub fn unwrap_attack(&self) -> Option<(&Vec<Vec2>, PlayerTile)> {
        match self {
            TileEvent::AttackEvent {
                ref targets,
                player_tile,
            } => Some((targets, *player_tile)),
            _ => None,
        }
    }

    pub fn unwrap_upgrade(&self) -> Option<(&Vec2, usize)> {
        match self {
            TileEvent::UpgradeEvent { ref target, hp } => Some((target, *hp)),
            _ => None,
        }
    }
}

#[derive(Event)]
pub struct GameOverEvent(pub Player);

fn register_event(
    (mouse, mut buttons): (Res<GridMouse>, ResMut<Input<MouseButton>>),
    mut event: EventWriter<TileEvent>,
    mut tile_query: Query<(&Position, &mut Owned, &mut Tile)>,
    attack_controller: Res<AttackController>,
    turn: Res<TurnCounter>,
    farms: Res<FarmCounter>,
    grid: Res<TileGrid>,
) -> Option<()> {
    const BUTTONS: [MouseButton; 2] = [MouseButton::Left, MouseButton::Right];
    let (button, owner, tile) = buttons
        .get_just_pressed()
        .find(|x| BUTTONS.contains(x))
        .and_then(|button| {
            tile_query
                .iter_mut()
                .find(|(pos, ..)| pos.as_grid_index() == mouse.grid_position())
                .map(|tile| (button, tile.1 .0, tile.2 .0))
        })?;

    let farms_available = farms.available_farms(turn.player());
    let targets = get_attack_targets(
        get_selected_grid_position(&attack_controller, &grid, &mouse, turn.player()),
        mouse.grid_position(),
        attack_controller.selected_level.unwrap_or(1),
    );

    info!("Targets: {:?}", targets);
    match (tile.player_tile(), owner, button, farms_available) {
        (Some(PlayerTile::Tile), Some(p), MouseButton::Right, _) if p == turn.player() => event
            .send(TileEvent::AttackEvent {
                targets,
                player_tile: PlayerTile::Farm,
            }),
        (Some(PlayerTile::Tile), Some(p), MouseButton::Left, 1..)
        | (Some(PlayerTile::Farm), Some(p), MouseButton::Left, _) // Farm upgrades are free (balancing TODO)
            if p == turn.player() => event.send(TileEvent::UpgradeEvent { target: mouse.grid_position(), hp: tile.is_tile() as usize} ),
        (.., MouseButton::Left, available @ 1..) if available >= targets.len() && !targets.is_empty() => event.send(TileEvent::AttackEvent {
            targets,
            player_tile: PlayerTile::Tile,
        }),
        _ => (),
    }
    buttons.clear();
    Some(())
}

fn select_tile(
    mouse: Res<GridMouse>,
    keys: Res<Input<KeyCode>>,
    mut attack_controller: ResMut<AttackController>,
    mut events: EventWriter<TileEvent>,
    mut tile_query: Query<(&Position, &Owned, &Tile, &Level)>,
    turn: Res<TurnCounter>,
) {
    keys.just_pressed(KeyCode::Space).then(|| {
        if attack_controller.selected.is_some() {
            attack_controller.deselect();
            events.send(TileEvent::DeselectEvent);
        } else if let Some(level) =
            tile_query
                .iter_mut()
                .find_map(|(pos, owner, Tile(tile_type), &Level(level))| {
                    (pos.as_grid_index() == mouse.grid_position()
                        && owner.0.is_some_and(|p| p == turn.player())
                        && tile_type.is_tile())
                    .then_some(level)
                })
        {
            attack_controller.select(mouse.grid_position(), level);
            events.send(TileEvent::SelectEvent(mouse.grid_position()));
        }
    });
}

fn upgrade(
    mut tile_query: Query<(&Position, &mut Level, &mut Health)>,
    mut events: EventReader<TileEvent>,
) -> Option<()> {
    events
        .read()
        .next()?
        .unwrap_upgrade()
        .and_then(|(&target, hp)| {
            tile_query
                .iter_mut()
                .find(|&(pos, ..)| pos.as_grid_index() == target)
                .map(|(_, mut level, mut health)| {
                    level.up();
                    health.0 += hp;
                })
        })
}

fn get_start_tuple(
    tile: TileType,
    player: Player,
    player_tile: PlayerTile,
) -> (TileType, usize, usize, Option<Player>) {
    (
        tile.with_player_tile(player_tile),
        1,
        tile.terrain().get_health(),
        Some(player),
    )
}

fn tile_attack(
    mut tile_query: Query<(&Position, &mut Tile, &mut Level, &mut Health, &mut Owned)>,
    mut events: EventReader<TileEvent>,
    turn: Res<TurnCounter>,
) -> Option<()> {
    events
        .read()
        .next()?
        .unwrap_attack()
        .and_then(|(targets, player_tile)| {
            (tile_query
                .iter_mut()
                .filter(|(pos, ..)| targets.contains(&pos.as_grid_index()))
                .map(|(_, mut tile, mut level, mut health, mut owner)| {
                    if health.damage() == 0 || player_tile == PlayerTile::Farm {
                        (tile.0, level.0, health.0, owner.0) =
                            get_start_tuple(tile.0, turn.player(), player_tile);
                    }
                })
                .count()
                > 0)
            .then_some(())
        })
}

fn delete_if_disconnected(
    mut tile_query: Query<(&Position, &mut Owned)>,
    grid: Res<TileGrid>,
    mut update_grid: EventWriter<GridUpdateEvent>,
) {
    tile_query
        .iter_mut()
        .filter(|query| !grid.is_connected_to_base(query))
        .for_each(|(_, mut owner)| {
            owner.0 = None;
        });
    update_grid.send(GridUpdateEvent);
}

fn check_win(owner_query: Query<&Owned>, mut game_over_event: EventWriter<GameOverEvent>) {
    let (red_owners, blue_owners): (Vec<_>, Vec<_>) = owner_query
        .iter()
        .filter(|owner| owner.0.is_some())
        .partition(|owner| owner.0 == Some(Player::Red));

    if red_owners.is_empty() {
        game_over_event.send(GameOverEvent(Player::Blue));
    } else if blue_owners.is_empty() {
        game_over_event.send(GameOverEvent(Player::Red));
    }
}

fn main_menu(
    mut state: ResMut<NextState<GameState>>,
    mut game_over: EventReader<GameOverEvent>,
    mut wins: ResMut<WinCounter>,
) {
    if let Some(&GameOverEvent(player)) = game_over.read().next() {
        wins.increment(player, 1);
        state.set(GameState::Menu);
    }
}

fn update_grid(
    mut refresh_query: Query<(
        &Position,
        &mut Handle<Image>,
        &Owned,
        &mut Health,
        &mut Tile,
        &mut Level,
    )>,
    assets: Res<TileAssets>,
    mut grid: ResMut<TileGrid>,
) {
    refresh_query.iter_mut().for_each(
        |(pos, mut image, &Owned(owner), mut health, mut tile, mut level)| {
            if owner.is_none() {
                tile.0.empty();
                health.0 = 0;
                level.0 = 0;
            }

            *image = assets.get(tile.0, level.0, owner);
            grid.set_owner(pos, owner);
        },
    );
}

#[derive(Component, Clone, Debug, PartialEq, Default)]
pub struct Selector;

fn update_selection(
    attack_controller: Res<AttackController>,
    mut selector_query: Query<(&mut Transform, &mut Visibility), With<Selector>>,
) {
    if let Some(selected_position) = attack_controller.selected {
        for (mut transform, mut visibility) in selector_query.iter_mut() {
            *visibility = Visibility::Visible;
            transform.translation =
                (selected_position * TILE_SIZE + Vec2::splat(TILE_SIZE / 2.0)).extend(0.0);
        }
    } else {
        for (_, mut visibility) in selector_query.iter_mut() {
            *visibility = Visibility::Hidden;
        }
    }
}
