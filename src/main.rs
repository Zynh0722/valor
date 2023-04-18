use std::sync::{Arc, Mutex};

use valor_lib::{watch_connection, ConnectionState};

struct AppState {
    connection: Arc<Mutex<ConnectionState>>,
}

fn main() {
    let state = AppState {
        connection: Arc::new(Mutex::new(ConnectionState::init())),
    };

    watch_connection(state.connection.clone()).unwrap();

    loop {}
}
