use std::{
    net::{SocketAddr, UdpSocket},
    thread,
    time::{Duration, Instant, SystemTime},
};

use bincode::ErrorKind;
use log::{info, warn};
use renet::{
    transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig}, ConnectionConfig, DefaultChannel, RenetServer, ServerEvent
};
use store::{ClientEvent, TileEvent};

const PROTOCOL_ID: u64 = 7;

fn main() {
    env_logger::init();

    let public_addr: SocketAddr = format!("0.0.0.0:{}", "5000").parse().unwrap();
    let connection_config = ConnectionConfig::default();
    let mut server: RenetServer = RenetServer::new(connection_config);

    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };
    let socket: UdpSocket = UdpSocket::bind(public_addr).unwrap();

    let mut transport = NetcodeServerTransport::new(server_config, socket).unwrap();

    let mut game_state = store::GameState::default();
    let mut last_updated = Instant::now();

    loop {
        let now = Instant::now();
        let duration = now - last_updated;
        last_updated = now;

        server.update(duration);
        transport.update(duration, &mut server).unwrap();

        while let Some(event) = server.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    info!("Player {} connected", client_id);

                    game_state.set_player_id(
                        client_id.raw(),
                        match server.connected_clients() {
                            1 => store::Player::Red,
                            2 => store::Player::Blue,
                            _ => unreachable!(),
                        },
                    );

                    println!("Player {} is {:?}", client_id, game_state.id_to_player.get(&client_id.raw()));

                    if server.connected_clients() == 2 {
                        server.broadcast_message(DefaultChannel::ReliableOrdered, bincode::serialize(&ClientEvent::Init(Box::new(game_state.grid.clone()))).unwrap());
                    }
                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    info!("Player {} disconnected: {}", client_id, reason);
                }
            }
        }

        for client_id in server.clients_id() {
            while let Some(message) = server.receive_message(client_id, DefaultChannel::ReliableOrdered) {
                bincode::deserialize::<TileEvent>(&message).map(|event| {
                    game_state.get_action(&event).map(|action| {
                        game_state.consume(&action).iter().for_each(|change| {
                            info!("Sending:\n\t{:#?}", change);
                            server.broadcast_message(DefaultChannel::ReliableOrdered, bincode::serialize(change).unwrap());
                        });
                    }).ok_or(Box::new(ErrorKind::Custom("Invalid action".to_string())))
                }).map_err(|err| {
                    warn!("Error: {}", err);
                }).ok();
            }
        }

        transport.send_packets(&mut server);
        thread::sleep(Duration::from_millis(10));
    }
}
