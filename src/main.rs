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
use grid_mouse::*;
use hud::HUDPlugin;
use menu::MenuPlugin;
use player::*;
use strum::IntoEnumIterator;
use terrain::TerrainPlugin;
use tiles::*;
use turn::*;
use utils::{attack_is_valid, get_rectified_mouse_position, get_vec_from_index};

mod assets;
mod camera;
mod consts;
mod debug;
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
    pub fn into_grid(&self) -> Vec2 {
        (self.0 - TILE_SIZE / 2.0) / TILE_SIZE
    }
}

impl Into<Transform> for Position {
    fn into(self) -> Transform {
        Transform::from_translation(self.0.extend(0.0))
    }
}

#[derive(Resource)]
pub struct AttackController {
    pub selected: Option<Vec2>,
    pub selected_level: usize,
}

impl Default for AttackController {
    fn default() -> Self {
        Self {
            selected: None,
            selected_level: 0,
        }
    }
}

impl AttackController {
    pub fn select(&mut self, position: Vec2, level: usize) {
        self.selected = Some(position);
        self.selected_level = level;
    }

    pub fn deselect(&mut self) {
        self.selected = None;
        self.selected_level = 0;
    }
}

#[derive(States, Reflect, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
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
            DebugPlugin,
            MenuPlugin,
            AssetsPlugin,
            TerrainPlugin,
        ))
        .add_event::<GridChangedEvent>()
        .add_event::<GridUpdateEvent>()
        .add_event::<TileEvent>()
        .insert_resource(TileGrid {
            grid: [(None, false); GRID_SIZE],
        })
        .insert_resource(AttackController::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                (
                    select_tile,
                    eat_tile,
                    create,
                    upgrade,
                    register_event,
                    update_image,
                    (delete_if_disconnected, update_grid).chain(),
                )
                    .run_if(in_state(GameState::Game)),
                close_on_esc,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, mut grid: ResMut<TileGrid>, assets: Res<TileAssets>) {
    (1..(MAP_HEIGHT * MAP_WIDTH * 4.0) as usize).for_each(|index| {
        if index == GRID_SIZE - 1 {
            return;
        }

        _ = commands.spawn(TileBundle::blank(get_vec_from_index(index), &assets))
    });

    Player::iter().for_each(|p| {
        commands.spawn(TileBundle::base(p, &assets));
        grid.make_base(p);
    });
}

#[derive(Event, Debug)]
pub(crate) struct TurnEvent;

#[derive(Event, Debug)]
pub enum TileEvent {
    CreateEvent(Vec2, PlayerTile),
    UpgradeEvent(Vec2, TileType),
    SelectEvent(Vec2),
    AttackEvent {
        origin: Vec2,
        target: Vec2,
        level: usize,
    },
}

fn register_event(
    mouse: Res<GridMouse>,
    mut buttons: ResMut<Input<MouseButton>>,
    mut event: EventWriter<TileEvent>,
    mut tile_query: Query<(&Position, &mut Owned, &mut Tile)>,
    attack_controller: Res<AttackController>,
    turn: Res<TurnCounter>,
) {
    if !buttons.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        return;
    }

    let button = buttons.get_just_pressed().nth(0).unwrap();
    let Some((_, owner, tile, ..)) = tile_query
        .iter_mut()
        .find(|(pos, ..)| pos.into_grid() == mouse.grid_position())
    else {
        return;
    };

    match (tile.0.player_tile(), owner.0, button) {
        (None, _, MouseButton::Left) => event.send(TileEvent::CreateEvent(
            mouse.grid_position(),
            PlayerTile::Tile,
        )),
        (Some(PlayerTile::Tile), Some(p), MouseButton::Right) if p == turn.player() => event.send(
            TileEvent::CreateEvent(mouse.grid_position(), PlayerTile::Farm),
        ),
        (Some(_), Some(p), MouseButton::Left) if p == turn.player() => {
            event.send(TileEvent::UpgradeEvent(mouse.grid_position(), tile.0))
        }
        (Some(_), Some(p), MouseButton::Left) if p != turn.player() => {
            event.send(TileEvent::AttackEvent {
                origin: attack_controller.selected.unwrap_or(mouse.grid_position()),
                target: mouse.grid_position(),
                level: attack_controller.selected_level,
            })
        }
        _ => (),
    }
    buttons.clear();
}

fn select_tile(
    mouse: Res<GridMouse>,
    keys: Res<Input<KeyCode>>,
    mut attack_controller: ResMut<AttackController>,
    mut events: EventWriter<TileEvent>,
    mut tile_query: Query<(&Position, &Owned, &Tile, &Level)>,
    mut update_image: EventWriter<UpdateImageEvent>,
    turn: Res<TurnCounter>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }

    let mouse_position = mouse.grid_position();

    let Some((_, _, Tile(TileType::Occupied(PlayerTile::Tile, _)), &Level(level), ..)) =
        tile_query.iter_mut().find(|(pos, owner, ..)| {
            pos.into_grid() == mouse_position && owner.0 == Some(turn.player())
        })
    else {
        return;
    };

    attack_controller.select(mouse_position, level);

    update_image.send(UpdateImageEvent);
    events.send(TileEvent::SelectEvent(mouse_position));
}

fn create(
    mut tile_query: Query<(
        &Position,
        &mut Owned,
        &mut Tile,
        &mut Level,
        &mut Health,
    )>,
    turn: Res<TurnCounter>,
    grid: ResMut<TileGrid>,
    mut events: EventReader<TileEvent>,
    mut update_image: EventWriter<UpdateImageEvent>,
    turn_event: EventWriter<TurnEvent>,
) {
    let Some(&TileEvent::CreateEvent(selected_position, player_tile)) = events.read().next() else {
        return;
    };

    if !grid.any_adjacent_tiles(selected_position, turn.player()) {
        return;
    }

    if let Some((_, owner, tile, level, health)) = tile_query
        .iter_mut()
        .find(|(pos, ..)| pos.into_grid() == selected_position)
    {
        info!("Creating tile at {:?}", selected_position);
        make_tile(
            level,
            tile,
            owner,
            health,
            grid,
            turn_event,
            selected_position,
            player_tile,
            turn,
        );

        update_image.send(UpdateImageEvent);
    }
}

fn make_tile(
    mut level: Mut<Level>,
    mut tile: Mut<Tile>,
    mut owner: Mut<Owned>,
    mut health: Mut<Health>,
    mut grid: ResMut<TileGrid>,
    mut turn_event: EventWriter<TurnEvent>,
    selected_position: Vec2,
    player_tile: PlayerTile,
    turn: Res<TurnCounter>,
) {
    level.0 = 1;
    tile.0.set_player_tile(player_tile);
    health.0 = match tile.0.terrain() {
        Terrain::Mountain => 2,
        Terrain::None => 1,
        _ => 0,
    };
    owner.0 = Some(turn.player());
    grid.set_owner(selected_position, Some(turn.player()));

    turn_event.send(TurnEvent);
}

fn upgrade(
    mut tile_query: Query<(&Position, &mut Level, &mut Health)>,
    mut events: EventReader<TileEvent>,
    mut turn_event: EventWriter<TurnEvent>,
    mut update_image: EventWriter<UpdateImageEvent>,
) {
    let Some(&TileEvent::UpgradeEvent(selected_position, tile)) = events.read().next() else {
        return;
    };

    if let Some((_, mut level, mut health)) = tile_query
        .iter_mut()
        .find(|&(pos, ..)| pos.into_grid() == selected_position)
    {
        level.up();

        if tile.is_tile() {
            health.0 += 1;
        }

        update_image.send(UpdateImageEvent);
        turn_event.send(TurnEvent);
    }
}

#[derive(Event, Debug)]
pub struct GridChangedEvent;

#[derive(Event, Debug)]
pub struct GridUpdateEvent;

#[derive(Event)]
pub struct UpdateImageEvent;

fn eat_tile(
    mut tile_query: Query<(
        &Position,
        &mut Owned,
        &mut Tile,
        &mut Level,
        &mut Health,
    )>,
    mut changes: EventWriter<GridChangedEvent>,
    mut events: EventReader<TileEvent>,
    mut turn_event: EventWriter<TurnEvent>,
    mut update_image: EventWriter<UpdateImageEvent>,
    turn: Res<TurnCounter>,
    grid: ResMut<TileGrid>,
) {
    let Some(&TileEvent::AttackEvent {
        origin,
        target,
        level: attacker_level,
    }) = events.read().next()
    else {
        return;
    };

    let me = turn.player();
    if !grid.any_adjacent_tiles(target, me) {
        return;
    }

    info!("Attacking tile at {:?} from {:?}", target, origin);

    if let Some((_, owner, tile, level, mut health)) = tile_query
        .iter_mut()
        .find(|(pos, ..)| pos.into_grid() == target)
    {
        if !attack_is_valid(origin, target, attacker_level) {
            return;
        }

        info!("Damaged tile at {:?} from {:?}", target, origin);

        health.damage();

        if health.0 > 0 {
            turn_event.send(TurnEvent);
            return;
        }

        make_tile(
            level,
            tile,
            owner,
            health,
            grid,
            turn_event,
            target,
            PlayerTile::Tile,
            turn,
        );

        update_image.send(UpdateImageEvent);
        changes.send(GridChangedEvent);
    }
}

fn delete_if_disconnected(
    mut tile_query: Query<(
        &Position,
        &mut Owned,
        &mut Tile,
        &mut Level,
    )>,
    grid: Res<TileGrid>,
    mut update_grid: EventWriter<GridUpdateEvent>,
    mut update_image: EventWriter<UpdateImageEvent>,
    mut changes: EventReader<GridChangedEvent>,
) {
    if changes.read().count() == 0 {
        return;
    }

    for (pos, mut owned, mut tile, mut level) in tile_query.iter_mut() {
        if owned.0.is_none() {
            continue;
        }

        if grid.is_connected_to_base(pos.clone().into_grid(), owned.0.unwrap()) {
            continue;
        } else {
            tile.0.empty();
            level.0 = 0;
            owned.0 = None;

            update_image.send(UpdateImageEvent);
            update_grid.send(GridUpdateEvent);
        }
    }
}

fn update_grid(
    mut tile_query: Query<(&Position, &Owned)>,
    mut grid: ResMut<TileGrid>,
    mut update: EventReader<GridUpdateEvent>,
) {
    if update.read().count() == 0 {
        return;
    }

    info!("Received change notification, updating grid");

    for (pos, owned) in tile_query.iter_mut() {
        grid.set_owner(pos.clone().into_grid(), owned.0);
    }
}

fn update_image(
    mut tile_query: Query<(&mut Handle<Image>, &Tile, &Level, &Owned)>,
    mut update_image: EventReader<UpdateImageEvent>,
    assets: Res<TileAssets>,
) {
    if update_image.read().count() == 0 {
        return;
    }

    for (mut image, tile, level, owner) in tile_query.iter_mut() {
        *image = assets.get(tile.0, level.0, owner.0);
    }
}