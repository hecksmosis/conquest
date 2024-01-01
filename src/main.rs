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
// use menu::MenuPlugin;
use player::*;
use strum::IntoEnumIterator;
use terrain::TerrainPlugin;
use tiles::*;
use turn::*;
use utils::{get_attack_targets, get_rectified_mouse_position, get_vec_from_index};

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
            // MenuPlugin,
            AssetsPlugin,
            TerrainPlugin,
        ))
        .add_event::<GridChangedEvent>()
        .add_event::<GridUpdateEvent>()
        .add_event::<TileEvent>()
        .add_event::<UpdateImageEvent>()
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
                    tile_attack,
                    upgrade,
                    register_event,
                    update_image.run_if(update_image_event),
                    update_selection,
                    (
                        delete_if_disconnected,
                        update_grid.run_if(grid_update_event),
                    )
                        .chain(),
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
}

fn grid_update_event(mut grid_update: EventReader<GridUpdateEvent>) -> bool {
    grid_update.read().count() == 0
}

fn update_image_event(mut update_image: EventReader<UpdateImageEvent>) -> bool {
    update_image.read().count() == 0
}

#[derive(Event)]
pub(crate) struct TurnEvent;

#[derive(Event)]
pub struct GridChangedEvent;

#[derive(Event)]
pub struct GridUpdateEvent;

#[derive(Event)]
pub struct UpdateImageEvent;

#[derive(Event, Debug)]
pub enum TileEvent {
    UpgradeEvent(Vec2, TileType),
    SelectEvent(Vec2),
    DeselectEvent,
    AttackEvent {
        origin: Option<Vec2>,
        target: Vec2,
        attacker_level: usize,
        player_tile: PlayerTile,
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

    let button = buttons.get_just_pressed().next().unwrap();
    let Some((_, owner, tile, ..)) = tile_query
        .iter_mut()
        .find(|(pos, ..)| pos.as_grid_index() == mouse.grid_position())
    else {
        return;
    };

    match (tile.0.player_tile(), owner.0, button) {
        (Some(PlayerTile::Tile), Some(p), MouseButton::Right) if p == turn.player() => {
            event.send(TileEvent::AttackEvent {
                origin: attack_controller.selected,
                target: mouse.grid_position(),
                attacker_level: attack_controller.selected_level.unwrap_or(1),
                player_tile: PlayerTile::Farm,
            })
        }
        (Some(_), Some(p), MouseButton::Left) if p == turn.player() => {
            event.send(TileEvent::UpgradeEvent(mouse.grid_position(), tile.0))
        }
        (.., MouseButton::Left) => event.send(TileEvent::AttackEvent {
            origin: attack_controller.selected,
            target: mouse.grid_position(),
            attacker_level: attack_controller.selected_level.unwrap_or(1),
            player_tile: PlayerTile::Tile,
        }),
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
    turn: Res<TurnCounter>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }

    let mouse_position = mouse.grid_position();

    let Some((_, _, Tile(TileType::Occupied(PlayerTile::Tile, _)), &Level(level), ..)) =
        tile_query.iter_mut().find(|(pos, owner, ..)| {
            pos.as_grid_index() == mouse_position && owner.0 == Some(turn.player())
        })
    else {
        return;
    };

    attack_controller.select(mouse_position, level);

    events.send(TileEvent::SelectEvent(mouse_position));
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
        .find(|&(pos, ..)| pos.as_grid_index() == selected_position)
    {
        level.up();

        if tile.is_tile() {
            health.0 += 1;
        }

        update_image.send(UpdateImageEvent);
        turn_event.send(TurnEvent);
    }
}

fn tile_attack(
    mut tile_query: Query<(&Position, &mut Owned, &mut Tile, &mut Level, &mut Health)>,
    mut events: EventReader<TileEvent>,
    mut changes: EventWriter<GridChangedEvent>,
    mut turn_event: EventWriter<TurnEvent>,
    mut update_image: EventWriter<UpdateImageEvent>,
    mut grid: ResMut<TileGrid>,
    turn: Res<TurnCounter>,
) {
    let Some(&TileEvent::AttackEvent {
        origin,
        target,
        attacker_level,
        player_tile,
    }) = events.read().next()
    else {
        return;
    };

    if origin.is_none() && !grid.any_adjacent_tiles(target, turn.player()) {
        return;
    }

    let origin = origin.unwrap_or(
        grid.get_connected_tiles(target, turn.player())
            .first()
            .map(|a| a.1)
            .unwrap_or(Vec2::MAX), // Will never be reached
    );
    let targets = get_attack_targets(origin, target, attacker_level);

    if targets.is_empty() {
        return;
    }

    info!("Attacking tile at {:?} from {:?}", target, origin);

    for (pos, mut owner, mut tile, mut level, mut health) in tile_query
        .iter_mut()
        .filter(|(pos, ..)| targets.contains(&pos.as_grid_index()))
    {
        health.damage();
        if health.0 > 0 {
            continue;
        }

        level.0 = 1;
        tile.0.set_player_tile(player_tile);
        health.0 = match tile.0.terrain() {
            Terrain::Mountain => 2,
            Terrain::None => 1,
            _ => 0,
        };
        owner.0 = Some(turn.player());
        grid.set_owner(pos.as_grid_index(), Some(turn.player()));
    }

    turn_event.send(TurnEvent);
    update_image.send(UpdateImageEvent);
    changes.send(GridChangedEvent);
}

fn delete_if_disconnected(
    mut tile_query: Query<(&Position, &mut Owned, &mut Tile, &mut Level)>,
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

        if grid.is_connected_to_base(pos.clone().as_grid_index(), owned.0.unwrap()) {
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

fn update_grid(mut tile_query: Query<(&Position, &Owned)>, mut grid: ResMut<TileGrid>) {
    for (pos, owned) in tile_query.iter_mut() {
        grid.set_owner(pos.clone().as_grid_index(), owned.0);
    }
}

fn update_image(
    mut tile_query: Query<(&mut Handle<Image>, &Tile, &Level, &Owned)>,
    assets: Res<TileAssets>,
) {
    for (mut image, tile, level, owner) in tile_query.iter_mut() {
        *image = assets.get(tile.0, level.0, owner.0);
    }
}

#[derive(Component, Clone, Debug, PartialEq, Default)]
pub struct Selector;

fn update_selection(
    attack_controller: Res<AttackController>,
    mut selector_query: Query<(&mut Transform, &mut Visibility), With<Selector>>,
) {
    let Some(selected_position) = attack_controller.selected else {
        for (_, mut visibility) in selector_query.iter_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    };

    for (mut transform, mut visibility) in selector_query.iter_mut() {
        *visibility = Visibility::Visible;
        transform.translation =
            (selected_position * TILE_SIZE + Vec2::splat(TILE_SIZE / 2.0)).extend(0.0);
    }
}
