use async_std::io::{self};
use async_std::net::{TcpListener, TcpStream};
use async_std::task;
use futures::channel::mpsc;
use futures::channel::mpsc::unbounded;
use log::*;

use async_std::sync::Arc;
use futures::{pin_mut, select, FutureExt, SinkExt, StreamExt};
use mc::connection::PostPacketAction;
use mc::connection::{ActiveComms, CommsRef, ConnectionState};
use mc::error::{McError, McResult};
use mc::game::{ClientMessage, Game};
use mc::server::ServerData;

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

async fn handle_client(
    mut game_tx: Sender<ClientMessage>,
    stream: io::Result<TcpStream>,
    server_data: Arc<ServerData>,
) -> McResult<()> {
    let stream = stream.map_err(McError::Io)?;
    let peer = stream.peer_addr().map_err(McError::Io)?;

    debug!("new client: {:?}", peer);
    let (clientbound_tx, mut clientbound_rx) = unbounded();
    // broker_tx
    //     .send(BrokerMessage::NewClient { stream: stream.clone(), peer })
    //     .await
    //     .map_err(McError::IoChannel)?;

    let (mut reader, mut writer, encryption) = {
        let (r, w) = (stream.clone(), stream);
        ActiveComms::new(r, w)
    };

    let comms = CommsRef::new(clientbound_tx.clone(), encryption);
    let mut connection = ConnectionState::new(comms);

    loop {
        let serverbound = async { reader.read_packet().await }.fuse();

        let clientbound = clientbound_rx.next().map(|p| p.expect("NONE?")).fuse();

        pin_mut!(serverbound, clientbound);

        match select! {
            // recvd a packet from client
            packet = serverbound => {
                match packet {
                    Err(e) => break Err(e),
                    Ok(packet) => connection.handle_packet(packet, &server_data).await,
                }
            },
            // got a packet in outgoing queue to send to client
            packet = clientbound => writer.send_packet(packet).await.map(|_| PostPacketAction::default()),
        } {
            Err(e) => {
                drop(serverbound);
                drop(clientbound);

                // flush clientbound queue
                while let Ok(Some(outgoing)) = clientbound_rx.try_next() {
                    let _ = writer.send_packet(outgoing).await;
                }

                break McResult::<()>::Err(e);
            }

            Ok(action) => match action {
                PostPacketAction::None => {}
                PostPacketAction::EnteredPlayState {
                    player_name,
                    player_uuid,
                } => {
                    game_tx
                        .send(ClientMessage::NewClient {
                            outgoing: clientbound_tx.clone(),
                            name: player_name,
                            uuid: player_uuid,
                        })
                        .await
                        .map_err(|_| McError::Sink)?;
                }
            },
        }
    }
}

async fn accept_clients(host: &str, port: u16, server_data: Arc<ServerData>) -> McResult<()> {
    let listener = TcpListener::bind((host, port)).await.map_err(McError::Io)?;
    info!("listening on {}:{}", host, port);

    // start broker task
    // let (broker_tx, broker_rx) = mpsc::unbounded();
    // let _ = task::spawn(run_broker(broker_rx));

    // start game
    let (game_tx, game_rx) = unbounded();

    let game = Game::new(game_rx);
    task::spawn(game.run());

    // start client loop
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        // let broker_tx = broker_tx.clone();
        let server_data = server_data.clone();
        let game_tx = game_tx.clone();

        let _ = task::spawn(async move {
            match handle_client(game_tx, stream, server_data).await {
                Err(McError::PleaseDisconnect) => debug!("politely closing connection"), // not an error
                Err(e) => error!("error handling client: {}", e),
                _ => {}
            }
        });
    }

    Ok(())
}

enum BrokerMessage {}

async fn run_broker(broker_rx: Receiver<BrokerMessage>) {
    // TODO broker will only access outgoing client queue, NOT the socket
    // let mut clients: Slab<Arc<TcpStream>> = Slab::with_capacity(16);
    /*
    let mut clients: HashMap<SocketAddr, Arc<TcpStream>> = HashMap::with_capacity(16);

    while let Some(msg) = broker_rx.next().await {
        match msg {
            BrokerMessage::NewClient { stream, peer } => {
                clients.insert(peer, stream);
            }
        }
    }
    */
}

pub fn main() {
    // TODO start server thread
    let server_data = Arc::new(ServerData::new().unwrap());

    let accept_future = accept_clients("127.0.0.1", 25565, server_data);
    if let Err(e) = task::block_on(accept_future) {
        error!("failed to run accept loop: {}", e);
        std::process::exit(1)
    }
}
