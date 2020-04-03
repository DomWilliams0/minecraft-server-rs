use log::*;
use mc::connection::Connection;
use mc::error::{McError, McResult};
use mc::field::VarIntField;
use mc::packet::PacketBody;
use std::io::{BufReader, Read};
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;

fn handle_connection(stream: TcpStream) {
    let peer = stream.peer_addr().unwrap();
    trace!("hello {:?}", peer);

    let mut connection = Connection::default();
    let mut reader = BufReader::new(stream);
    let mut buf = Vec::with_capacity(8 * 1024);

    let mut do_handle = |connection: Connection| -> McResult<Connection> {
        let length = VarIntField::read(&mut reader)?.value();
        if length < 1 || length > 65535 {
            return Err(McError::BadPacketLength(length as usize));
        }

        let packet_id = VarIntField::read(&mut reader)?.value();

        buf.clear();
        buf.resize(length as usize, 0);
        if length > 0 {
            reader.read_exact(&mut buf).map_err(McError::Io)?;
        }

        connection.handle(PacketBody {
            id: packet_id,
            body: &buf,
        })
    };

    loop {
        connection = match do_handle(connection) {
            Err(e) => {
                warn!("error handling client: {:?}", e);
                break;
            }
            Ok(c) => c,
        };
    }

    trace!("goodbye {:?}", peer);
}

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let addr = ("127.0.0.1", 25565);
    let listener = TcpListener::bind(addr).unwrap();
    info!("listening on {}:{}", addr.0, addr.1);

    for stream in listener.incoming() {
        debug!("new connection!");
        spawn(move || {
            handle_connection(stream.unwrap());
        });
    }
}
