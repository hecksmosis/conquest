#![allow(clippy::type_complexity)]

use assets::{AssetsPlugin, TileAssets};
pub use bevy::prelude::*;
use bevy::{
    utils::HashMap,
    window::{close_on_esc, PrimaryWindow},
};
use bevy_renet::{
    renet::{
        transport::{ClientAuthentication, NetcodeClientTransport, NetcodeTransportError},
        ConnectionConfig, DefaultChannel, RenetClient,
    },
    transport::NetcodeClientPlugin,
    RenetClientPlugin,
};
use bevy_simple_text_input::TextInputPlugin;
use camera::CameraPlugin;
use grid_mouse::*;
use hud::{FarmText, HUDPlugin, PlacementModeText, TurnText};
use menu::MenuPlugin;
use std::{net::UdpSocket, time::SystemTime};
use store::*;
use tiles::*;
use utils::{get_rectified_mouse_position, get_vec_from_index};

mod assets;
mod camera;
mod grid_mouse;
mod hud;
mod menu;
mod tiles;
mod utils;

const PROTOCOL_ID: u64 = 7;

fn main() {
    let mut app = App::new();

    app.add_state::<ClientState>();
    app.add_plugins((
        DefaultPlugins,
        CameraPlugin,
        GridMousePlugin,
        HUDPlugin,
        MenuPlugin,
        AssetsPlugin,
        RenetClientPlugin,
        NetcodeClientPlugin,
        TextInputPlugin,
    ));
    app.add_event::<TileEvent>().add_event::<ClientEvent>();

    app.insert_resource(EntityTable {
        tiles: HashMap::default(),
        selector: None,
    });

    app.add_systems(OnEnter(ClientState::Lobby), insert_client)
        .add_systems(OnEnter(ClientState::Terrain), setup.after(menu::cleanup))
        .add_systems(OnExit(ClientState::Game), cleanup)
        .add_systems(
            Update,
            (
                register_event
                    .map(noop)
                    .run_if(in_state(ClientState::Game).or_else(in_state(ClientState::Terrain))),
                receive_events_from_server
                    .run_if(in_state(ClientState::Game).or_else(in_state(ClientState::Terrain))),
                start_game.run_if(in_state(ClientState::Lobby)),
                panic_on_error_system,
                close_on_esc,
            ),
        );

    app.run();
}

#[derive(Resource)]
pub struct EntityTable {
    pub tiles: HashMap<usize, Entity>,
    pub selector: Option<Entity>,
}

fn insert_client(world: &mut World) {
    let (client, transport) = new_renet_client();
    world.insert_resource(client);
    world.insert_resource(transport);
}

fn start_game(mut client: ResMut<RenetClient>, mut next_state: ResMut<NextState<ClientState>>) {
    if let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        let event: StartGame = bincode::deserialize(&message).unwrap();
        info!("{:#?}", event);

        next_state.set(ClientState::Terrain);
    }
}

fn new_renet_client() -> (RenetClient, NetcodeClientTransport) {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
    let client = RenetClient::new(ConnectionConfig::default());

    (client, transport)
}

fn setup(mut commands: Commands, mut entity_table: ResMut<EntityTable>, assets: Res<TileAssets>) {
    (0..(MAP_HEIGHT * MAP_WIDTH * 4.0) as usize).for_each(|index| {
        entity_table.tiles.insert(
            index,
            commands
                .spawn(TileBundle::blank(get_vec_from_index(index), &assets))
                .id(),
        );
    });

    // Spawn selector
    entity_table.selector = Some(
        commands
            .spawn((
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
            ))
            .id(),
    );
}

fn cleanup(mut commands: Commands, mut entities: Query<Entity, With<Selector>>) {
    for e in entities.iter_mut() {
        commands.entity(e).despawn_recursive();
    }
}

fn noop<T>(_: T) {}

fn register_event(
    (mouse, mut buttons): (Res<GridMouse>, ResMut<Input<MouseButton>>),
    keys: Res<Input<KeyCode>>,
    (mut client, transport): (ResMut<RenetClient>, Res<NetcodeClientTransport>),
    state: Res<State<ClientState>>,
) -> Option<()> {
    const INPUTS: [GameInput; 6] = [
        GameInput::Mouse(MouseButton::Left),
        GameInput::Mouse(MouseButton::Right),
        GameInput::Keyboard(KeyCode::Space),
        GameInput::Keyboard(KeyCode::M),
        GameInput::Keyboard(KeyCode::W),
        GameInput::Keyboard(KeyCode::Return),
    ];
    let keys = keys.get_just_pressed().map(|k| GameInput::Keyboard(*k));
    let input = buttons
        .get_just_pressed()
        .map(|b| GameInput::Mouse(*b))
        .chain(keys)
        .find(|x| INPUTS.contains(x))?;
    info!("{:?}", mouse.grid_position());
    client.send_message(
        DefaultChannel::ReliableOrdered,
        bincode::serialize(&TileEvent::from_input(
            transport.client_id(),
            mouse.grid_position(),
            input,
            state.get(),
        ))
        .unwrap(),
    );

    buttons.clear();
    Some(())
}

#[allow(clippy::too_many_arguments)]
fn receive_events_from_server(
    mut client: ResMut<RenetClient>,
    mut tiles: Query<(&Position, &mut Handle<Image>)>,
    mut farms_text: Query<
        &mut Text,
        (
            With<FarmText>,
            Without<TurnText>,
            Without<PlacementModeText>,
        ),
    >,
    mut turn_text: Query<
        &mut Text,
        (
            With<TurnText>,
            Without<FarmText>,
            Without<PlacementModeText>,
        ),
    >,
    mut terrain_text: Query<
        &mut Text,
        (
            With<PlacementModeText>,
            Without<FarmText>,
            Without<TurnText>,
        ),
    >,
    mut commands: Commands,
    mut next_state: ResMut<NextState<ClientState>>,
    entity_table: Res<EntityTable>,
    assets: Res<TileAssets>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        let event: ClientEvent = bincode::deserialize(&message).unwrap();
        info!("{:#?}", event);

        match event {
            ClientEvent::Init(grid) => {
                for (pos, mut image) in tiles.iter_mut() {
                    *image = assets.get(grid.get_tile(pos.as_grid_index()));
                }

                turn_text.iter_mut().for_each(|mut t| {
                    t.sections[1].value = "red".into();
                    t.sections[1].style.color = Color::RED;
                });

                terrain_text.iter_mut().for_each(|mut t| {
                    t.sections[1].value = "Mountain".into();
                });

                farms_text.iter_mut().for_each(|mut t| {
                    t.sections[1].value = "0".into();
                });
            }
            ClientEvent::TileChanges(changes) => changes.iter().for_each(|change| {
                if let Some((_, mut image)) = tiles
                    .iter_mut()
                    .find(|(pos, _)| pos.as_grid_index() == change.position)
                {
                    *image = assets.get(change.tile);
                }
            }),
            ClientEvent::Select(position) => {
                if let Some(e) = entity_table.selector {
                    commands.entity(e).insert((
                        Visibility::Visible,
                        Transform::from_translation(
                            (position * TILE_SIZE + Vec2::splat(TILE_SIZE / 2.0)).extend(0.0),
                        ),
                    ));
                }
            }
            ClientEvent::Deselect => {
                if let Some(e) = entity_table.selector {
                    commands.entity(e).insert(Visibility::Hidden);
                }
            }
            ClientEvent::Farms(farms) => {
                farms_text.iter_mut().enumerate().for_each(|(i, mut t)| {
                    t.sections[1].value = format!("{}", farms[i]);
                });
            }
            ClientEvent::Turn(player) => turn_text.iter_mut().for_each(|mut t| {
                t.sections[1].value = format!("{}", player);
                t.sections[1].style.color = match player {
                    Player::Red => Color::RED,
                    Player::Blue => Color::BLUE,
                };
            }),
            ClientEvent::TerrainMode(terrain) => terrain_text.iter_mut().for_each(|mut t| {
                t.sections[1].value = format!("{}", terrain);
            }),
            ClientEvent::GamePhase(state) => next_state.set(state),
        }
    }
}

#[derive(Component, Clone, Debug, PartialEq, Default)]
pub struct Selector;

// If any error is found we just panic
#[allow(clippy::never_loop)]
fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
    for e in renet_error.read() {
        panic!("{}", e);
    }
}
