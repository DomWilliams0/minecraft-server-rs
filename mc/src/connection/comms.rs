use crate::error::{McError, McResult};
use cryptostream::read::Decryptor;
use cryptostream::write::Encryptor;

use openssl::symm::Cipher;
use std::io::{Read, Write};

use std::net::TcpStream;

pub trait Stream: Read + Write + Sized {
    fn try_clone(&self) -> McResult<Self>;
}

impl Stream for TcpStream {
    fn try_clone(&self) -> McResult<Self> {
        self.try_clone().map_err(McError::Io)
    }
}

pub(crate) enum ActiveComms<S: Stream> {
    Plaintext {
        stream: S,
    },
    Encrypted {
        reader: Decryptor<S>,
        writer: Encryptor<S>,
    },
}

impl<S: Stream> ActiveComms<S> {
    pub fn new(stream: S) -> Self {
        ActiveComms::Plaintext { stream }
    }
}

impl<S: Stream> Read for ActiveComms<S> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            ActiveComms::Plaintext { stream } => stream.read(buf),
            ActiveComms::Encrypted { reader, .. } => reader.read(buf),
        }
    }
}

impl<S: Stream> Write for ActiveComms<S> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            ActiveComms::Plaintext { stream } => stream.write(buf),
            ActiveComms::Encrypted { writer, .. } => writer.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            ActiveComms::Plaintext { stream } => stream.flush(),
            ActiveComms::Encrypted { writer, .. } => writer.flush(),
        }
    }
}

impl<S: Stream> ActiveComms<S> {
    pub fn upgrade(&mut self, shared_secret: Vec<u8>) -> McResult<()> {
        if let ActiveComms::Plaintext { stream } = self {
            let r = stream.try_clone()?;
            let w = stream.try_clone()?;

            let cipher = Cipher::aes_128_cfb8();
            *self = ActiveComms::Encrypted {
                reader: Decryptor::new(r, cipher, &shared_secret, &shared_secret)
                    .map_err(McError::OpenSSL)?,
                writer: Encryptor::new(w, cipher, &shared_secret, &shared_secret)
                    .map_err(McError::OpenSSL)?,
            };
        }

        Ok(())
    }
}
