use std::net::{TcpListener, TcpStream};
use std::thread::spawn;

use log::*;

use mc::connection::ConnectionState;
use mc::error::McError;
use mc::server::{ServerData, ServerDataRef};

fn handle_connection(server_data: ServerDataRef, stream: TcpStream) {
    let peer = stream.peer_addr().unwrap();
    trace!("hello {:?}", peer);

    // TODO bufstream?
    let mut connection = ConnectionState::new(server_data, stream);

    loop {
        match connection.handle_transaction() {
            Err(McError::PleaseDisconnect) => {
                debug!("closing connection");
                break;
            }
            Err(e) => {
                warn!("error handling client: {:?}", e);
                break;
            }
            Ok(_) => {}
        };
    }

    trace!("goodbye {:?}", peer);
}

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Trace)
        .init();

    let server_data = ServerData::new().unwrap_or_else(|e| {
        error!("failed to init server: {:?}", e);
        std::process::exit(1)
    });

    let addr = ("127.0.0.1", 25565);
    let listener = TcpListener::bind(addr).unwrap();
    info!("listening on {}:{}", addr.0, addr.1);

    for stream in listener.incoming() {
        debug!("new connection!");
        let server_data = server_data.clone();
        spawn(move || {
            handle_connection(server_data, stream.unwrap());
        });
    }
}
