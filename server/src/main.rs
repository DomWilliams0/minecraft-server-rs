use log::*;
use mc::connection::Connection;
use mc::error::{McError, McResult};
use mc::field::{Field, VarIntField};
use mc::packet::PacketBody;
use std::io::{ErrorKind, Read};
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;

fn handle_transaction(
    conn: Connection,
    stream: TcpStream,
    buf: &mut Vec<u8>,
) -> McResult<Option<(Connection, TcpStream)>> {
    let mut bufstream = bufstream::BufStream::with_capacities(8192, 8192, stream);

    let mut length = match VarIntField::read(&mut bufstream) {
        Err(McError::Io(e)) if e.kind() == ErrorKind::UnexpectedEof => {
            debug!("eof");
            return Ok(None);
        }
        Err(e) => return Err(e),
        Ok(len) => len.value(),
    };

    if length < 1 || length > 65535 {
        return Err(McError::BadPacketLength(length as usize));
    }

    debug!("packet length={}", length);

    let packet_id = {
        let varint = VarIntField::read(&mut bufstream)?;
        length -= varint.size() as i32; // length includes packet id
        varint.value()
    };

    debug!("packet id={:#x}", packet_id);

    buf.clear();
    buf.resize(length as usize, 0);
    if length > 0 {
        bufstream.read_exact(buf).map_err(McError::Io)?;
    }

    let new_connection = conn.handle(
        PacketBody {
            id: packet_id,
            body: &buf,
        },
        &mut bufstream,
    )?;

    // flush stream
    let stream = bufstream
        .into_inner()
        .map_err(|e| McError::StreamFlush(e.to_string()))?;
    Ok(Some((new_connection, stream)))
}

fn handle_connection(mut stream: TcpStream) {
    let peer = stream.peer_addr().unwrap();
    trace!("hello {:?}", peer);

    let mut connection = Connection::default();
    let mut buf = Vec::with_capacity(8 * 1024);

    loop {
        match handle_transaction(connection, stream, &mut buf) {
            Err(e) => {
                warn!("error handling client: {:?}", e);
                break;
            }
            Ok(None) => break, // eof
            Ok(Some((Connection::Disconnect, _))) => {
                debug!("closing connection");
                break;
            }
            Ok(Some((c, s))) => {
                connection = c;
                stream = s;
            }
        };
    }

    trace!("goodbye {:?}", peer);
}

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Trace)
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
