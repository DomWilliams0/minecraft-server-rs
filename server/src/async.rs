use async_std::prelude::*;
use log::*;

use async_std::io;
use async_std::net::{TcpListener, TcpStream};
use async_std::task;
use futures::channel::mpsc;
use mc::connection::ConnectionState;
use mc::error::{McError, McResult};
use mc::server::{ServerData, ServerDataRef};

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

async fn handle_client(
    broker_tx: Sender<BrokerMessage>,
    stream: io::Result<TcpStream>,
    server_data: ServerDataRef,
) -> McResult<()> {
    let stream = stream.map_err(McError::Io)?;
    let peer = stream.peer_addr().map_err(McError::Io)?;

    debug!("new client: {:?}", peer);
    // broker_tx
    //     .send(BrokerMessage::NewClient { stream: stream.clone(), peer })
    //     .await
    //     .map_err(McError::IoChannel)?;

    let mut state = ConnectionState::new(server_data, stream);

    loop {
        state.handle_transaction().await?;
    }
}

async fn accept_clients(host: &str, port: u16, server_data: ServerDataRef) -> McResult<()> {
    let listener = TcpListener::bind((host, port)).await.map_err(McError::Io)?;
    info!("listening on {}:{}", host, port);

    // start broker task
    let (broker_tx, broker_rx) = mpsc::unbounded();
    let _ = task::spawn(run_broker(broker_rx));

    // start client loop
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let broker_tx = broker_tx.clone();
        let server_data = server_data.clone();
        let _ = task::spawn(async move {
            match handle_client(broker_tx, stream, server_data).await {
                Err(McError::PleaseDisconnect) => debug!("politely closing connection"), // not an error
                Err(e) => error!("error handling client: {}", e),
                _ => {}
            }
        });
    }

    Ok(())
}

enum BrokerMessage {}

async fn run_broker(mut broker_rx: Receiver<BrokerMessage>) {
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
    let server_data = ServerData::new().unwrap();

    let accept_future = accept_clients("127.0.0.1", 25565, server_data);
    if let Err(e) = task::block_on(accept_future) {
        error!("failed to run accept loop: {}", e);
        std::process::exit(1)
    }
}
