use crate::field::*;
use crate::packet::{ClientBound, ClientBoundPacket, PacketBody};
use crate::prelude::*;
use async_std::io::ErrorKind;
use async_std::sync::{Arc, RwLock};
use futures::{Sink, SinkExt};
use openssl::symm::{encrypt, Cipher};
use std::ops::Deref;

#[async_trait]
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

    pub(crate) async fn send_response<P: ClientBound + Sync + Send>(
        &mut self,
        packet: P,
    ) -> McResult<()> {
        let c = self.serialize_packet(packet).await?;
        self.response_sink.send(c).await.map_err(|_| McError::Sink)
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

    async fn serialize_packet<P: ClientBound + Sync + Send>(
        &mut self,
        packet: P,
    ) -> McResult<ClientBoundPacket> {
        let enc = self.encryption.read().await;
        let plaintext = ClientBoundPacket::from(packet);

        match enc.deref() {
            Encryption::Plaintext => Ok(plaintext),
            Encryption::Encrypted {
                shared_secret,
                cipher,
            } => {
                // TODO use Frame to encrypt a stream instead of double allocing
                encrypt(*cipher, &shared_secret, Some(&shared_secret), &plaintext)
                    .map(ClientBoundPacket::from)
                    .map_err(McError::OpenSSL)
            }
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

    /// Should already be encrypted
    pub async fn send_packet(&mut self, packet: ClientBoundPacket) -> McResult<()> {
        // let enc = self.encryption.read().await;
        // let blob = enc.serialize_packet(packet)?;
        self.stream.write_all(&packet).await.map_err(McError::Io)
    }

    //noinspection RsUnresolvedReference - idk why read_exact isn't found by the IDE
    pub async fn read_packet(&mut self) -> McResult<PacketBody> {
        // TODO DECRYPT STREAM
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
