use std::pin::Pin;

use async_std::task::{Context, Poll};

use crate::prelude::*;

// TODO delegate?
pub(crate) enum ActiveComms<S: McStream> {
    Plaintext { stream: S },
    // Encrypted {
    //     reader: Decryptor<S>,
    //     writer: Encryptor<S>,
    // },
}

impl<S: McStream> ActiveComms<S> {
    pub fn new(stream: S) -> Self {
        ActiveComms::Plaintext { stream }
    }
}

impl<S: McStream> Read for ActiveComms<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<IoResult<usize>> {
        match self.get_mut() {
            ActiveComms::Plaintext { stream } => Pin::new(stream).poll_read(cx, buf),
            // ActiveComms::Encrypted { reader, .. } => reader.read(buf),
        }
    }
}

impl<S: McStream> Write for ActiveComms<S> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<IoResult<usize>> {
        match self.get_mut() {
            ActiveComms::Plaintext { stream } => Pin::new(stream).poll_write(cx, buf),
            // ActiveComms::Encrypted { reader, .. } => reader.read(buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        match self.get_mut() {
            ActiveComms::Plaintext { stream } => Pin::new(stream).poll_flush(cx),
            // ActiveComms::Encrypted { reader, .. } => reader.read(buf),
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        match self.get_mut() {
            ActiveComms::Plaintext { stream } => Pin::new(stream).poll_close(cx),
            // ActiveComms::Encrypted { reader, .. } => reader.read(buf),
        }
    }
}
//
// impl<S: McStream> Write for ActiveComms<S> {
//     fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
//         match self {
//             ActiveComms::Plaintext { stream } => stream.write(buf),
//             // ActiveComms::Encrypted { writer, .. } => writer.write(buf),
//         }
//     }
//
//     fn flush(&mut self) -> std::io::Result<()> {
//         match self {
//             ActiveComms::Plaintext { stream } => stream.flush(),
//             // ActiveComms::Encrypted { writer, .. } => writer.flush(),
//         }
//     }
// }

impl<S: McStream> ActiveComms<S> {
    pub fn upgrade(&mut self, shared_secret: Vec<u8>) -> McResult<()> {
        // if let ActiveComms::Plaintext { stream } = self {
        //     let r = stream.try_clone()?;
        //     let w = stream.try_clone()?;
        //
        //     let cipher = Cipher::aes_128_cfb8();
        //     *self = ActiveComms::Encrypted {
        //         reader: Decryptor::new(r, cipher, &shared_secret, &shared_secret)
        //             .map_err(McError::OpenSSL)?,
        //         writer: Encryptor::new(w, cipher, &shared_secret, &shared_secret)
        //             .map_err(McError::OpenSSL)?,
        //     };
        // }
        //
        // Ok(())
        todo!()
    }
}
