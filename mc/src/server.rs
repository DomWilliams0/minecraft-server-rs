use openssl::pkey::Private;
use openssl::rsa::{Padding, Rsa};

use crate::config;
use crate::error::{McError, McResult};

// TODO don't need to wrap the whole struct in mutex?
// pub type &ServerData = Arc<Mutex<ServerData>>;

pub enum OnlineStatus {
    Online { public_key: Vec<u8> },
    Offline,
}

pub struct ServerData {
    rsa_key: Rsa<Private>,
}

impl ServerData {
    pub fn new() -> McResult<Self> {
        Ok(Self {
            // TODO only generate if online
            rsa_key: Rsa::generate(1024).map_err(McError::OpenSSL)?,
        })

        // Ok(Arc::new(Mutex::new(data)))
    }

    pub fn public_key(&self) -> McResult<Vec<u8>> {
        self.rsa_key.public_key_to_der().map_err(McError::OpenSSL)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> McResult<Vec<u8>> {
        let mut plaintext = vec![0u8; ciphertext.len()];
        let length = self
            .rsa_key
            .private_decrypt(ciphertext, &mut plaintext, Padding::PKCS1)
            .map_err(McError::OpenSSL)?;
        plaintext.truncate(length);
        Ok(plaintext)
    }

    pub fn online_status(&self) -> McResult<OnlineStatus> {
        if config::ONLINE_MODE {
            Ok(OnlineStatus::Online {
                public_key: self.public_key()?,
            })
        } else {
            Ok(OnlineStatus::Offline)
        }
    }

    // pub fn client_data_or_new(&mut self) -> &mut ClientData {
    //     let tid = std::thread::current().id();
    //     self.client_data.entry(tid).or_default()
    // }
    //
    // pub fn client_data(&mut self) -> McResult<&mut ClientData> {
    //     let tid = std::thread::current().id();
    //     self.client_data.get_mut(&tid).ok_or(McError::MissingClientData)
    // }
    //
    // pub fn remove_client_data(&mut self)  {
    //     let tid = std::thread::current().id();
    //     self.client_data.remove(&tid);
    // }
}
