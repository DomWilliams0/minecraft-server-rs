use async_std::io::{self};
use async_std::net::{TcpListener, TcpStream};
use async_std::task;
use futures::channel::mpsc;
use futures::channel::mpsc::unbounded;
use log::*;

use async_std::sync::Arc;
use futures::{pin_mut, select, FutureExt, SinkExt, StreamExt};
use mc::connection::{ActiveComms, CommsRef, ConnectionState};
use mc::error::{McError, McResult};
use mc::packet::ClientBoundPacket;
use mc::server::ServerData;

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

async fn handle_client(
    clientbound_tx: Sender<ClientBoundPacket>,
    mut clientbound_rx: Receiver<ClientBoundPacket>,
    stream: io::Result<TcpStream>,
    server_data: Arc<ServerData>,
) -> McResult<()> {
    let stream = stream.map_err(McError::Io)?;
    let peer = stream.peer_addr().map_err(McError::Io)?;

    debug!("new client: {:?}", peer);
    // broker_tx
    //     .send(BrokerMessage::NewClient { stream: stream.clone(), peer })
    //     .await
    //     .map_err(McError::IoChannel)?;

    let (mut reader, mut writer, encryption) = {
        let (r, w) = (stream.clone(), stream);
        ActiveComms::new(r, w)
    };

    let comms = CommsRef::new(clientbound_tx, encryption);
    let mut connection = ConnectionState::new(comms);

    loop {
        let serverbound = async { reader.read_packet().await }.fuse();

        let clientbound = clientbound_rx.next().map(|p| p.expect("NONE?")).fuse();

        pin_mut!(serverbound, clientbound);

        if let Err(e) = select! {
            // recvd a packet from client
            packet = serverbound => {
                match packet {
                    Err(e) => break Err(e),
                    Ok(packet) => connection.handle_packet(packet, &server_data).await,
                }
            },
            // got a packet in outgoing queue to send to client
            packet = clientbound => writer.send_packet(packet).await,
        } {
            drop(serverbound);
            drop(clientbound);

            // flush clientbound queue
            while let Ok(Some(outgoing)) = clientbound_rx.try_next() {
                let _ = writer.send_packet(outgoing).await;
            }

            break McResult::<()>::Err(e);
        }
    }
}

async fn accept_clients(host: &str, port: u16, server_data: Arc<ServerData>) -> McResult<()> {
    let listener = TcpListener::bind((host, port)).await.map_err(McError::Io)?;
    info!("listening on {}:{}", host, port);

    // start broker task
    // let (broker_tx, broker_rx) = mpsc::unbounded();
    // let _ = task::spawn(run_broker(broker_rx));

    // start client loop
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        // let broker_tx = broker_tx.clone();
        let server_data = server_data.clone();
        let (clientbound_tx, clientbound_rx) = unbounded();

        // TODO keep clientbound_tx for server

        let _ = task::spawn(async move {
            match handle_client(clientbound_tx, clientbound_rx, stream, server_data).await {
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
