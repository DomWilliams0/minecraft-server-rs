use crate::field::*;
use crate::packet::{ClientBound, ClientBoundPacket, PacketBody};
use crate::prelude::*;
use async_std::io::{Cursor, ErrorKind};
use async_std::sync::{Arc, RwLock};
use futures::{Sink, SinkExt};
use openssl::symm::{encrypt, Cipher};

pub trait ResponseSink: Sink<ClientBoundPacket> + Unpin + Send + Sync {}

impl<S: Sink<ClientBoundPacket> + Unpin + Send + Sync> ResponseSink for S {}

pub enum Encryption {
    Plaintext,
    Encrypted {
        shared_secret: Vec<u8>,
        cipher: Cipher,
    },
}

pub type CommsEncryption = Arc<RwLock<Encryption>>;

pub struct ActiveComms<S: McStream> {
    encryption: CommsEncryption,
    stream: S,
}

pub struct CommsRef<R: ResponseSink> {
    response_sink: R,
    encryption: CommsEncryption,
}
impl<R: ResponseSink> CommsRef<R> {
    pub fn new(response_sink: R, encryption: CommsEncryption) -> Self {
        Self {
            response_sink,
            encryption,
        }
    }

    pub(crate) async fn send_response<P: ClientBound + 'static>(
        &mut self,
        packet: P,
    ) -> McResult<()> {
        self.response_sink
            .send(packet.into())
            .await
            .map_err(|_| McError::Sink)
    }

    pub async fn upgrade(&self, shared_secret: Vec<u8>) {
        let mut guard = self.encryption.write().await;
        *guard = Encryption::Encrypted {
            shared_secret,
            cipher: Cipher::aes_128_cfb8(),
        }
    }

    pub async fn close(&mut self) -> McResult<()> {
        self.response_sink.close().await.map_err(|_| McError::Sink)
    }
}

impl Encryption {
    async fn serialize_packet(&self, packet: ClientBoundPacket) -> McResult<Box<[u8]>> {
        let plaintext = {
            let mut buf = vec![0u8; packet.full_size()];
            let mut cursor = Cursor::new(buf.as_mut_slice());
            packet.write_packet(&mut cursor).await?;
            buf
        };

        match self {
            Encryption::Plaintext => Ok(plaintext.into_boxed_slice()),
            Encryption::Encrypted {
                shared_secret,
                cipher,
            } => encrypt(*cipher, &shared_secret, Some(&shared_secret), &plaintext)
                .map(Vec::into_boxed_slice)
                .map_err(McError::OpenSSL),
        }
    }
}

impl<S: McStream> ActiveComms<S> {
    pub fn new(reader: S, writer: S) -> (Self, Self, CommsEncryption) {
        let enc = Arc::new(RwLock::new(Encryption::Plaintext));

        let r = Self {
            encryption: enc.clone(),
            stream: reader,
        };

        let w = Self {
            encryption: enc.clone(),
            stream: writer,
        };

        (r, w, enc)
    }

    pub async fn send_packet(&mut self, packet: ClientBoundPacket) -> McResult<()> {
        // let enc = self.encryption.read().await;
        // let blob = enc.serialize_packet(packet)?;
        // TODO streamify?
        let enc = self.encryption.read().await;
        let serialized = enc.serialize_packet(packet).await?;
        self.stream
            .write_all(&serialized)
            .await
            .map_err(McError::Io)
    }

    //noinspection RsUnresolvedReference - idk why read_exact isn't found by the IDE
    pub async fn read_packet(&mut self) -> McResult<PacketBody> {
        // TODO DECRYPT STREAM?!
        let mut length = match VarIntField::read_field(&mut self.stream).await {
            Err(McError::Io(e)) if e.kind() == ErrorKind::UnexpectedEof => {
                debug!("eof");
                return Err(McError::PleaseDisconnect);
            }

            Err(e) => return Err(e),
            Ok(len) => len.value(),
        };

        if length < 1 || length > 65535 {
            return Err(McError::BadPacketLength(length as usize));
        }

        debug!("packet length={}", length);

        let packet_id = {
            let varint = VarIntField::read_field(&mut self.stream).await?;
            length -= varint.size() as i32; // length includes packet id
            varint.value()
        };

        debug!("packet id={:#x}", packet_id);

        let mut recv_buf = vec![0u8; length as usize]; // TODO somehow reuse a buffer in self without making borrowck shit itself
        if length > 0 {
            self.stream
                .read_exact(&mut recv_buf)
                .await
                .map_err(McError::Io)?;
        }

        Ok(PacketBody {
            id: packet_id,
            body: recv_buf,
        })
    }
}
