use std::sync::{Arc, Mutex};

pub type ServerDataRef = Arc<Mutex<ServerData>>;

pub struct ServerData {
    // TODO enc keys
// TODO per-client data
}

impl ServerData {
    pub fn new() -> ServerDataRef {
        let data = Self {};

        Arc::new(Mutex::new(data))
    }
}
