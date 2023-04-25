use std::{sync::{Arc, Mutex}, time::Duration};

use notify::RecommendedWatcher;
use valor_lib::{watch_connection, ConnectionState};

#[derive(Debug)]
struct AppState {
    connection: Arc<Mutex<ConnectionState>>,
    watcher: Arc<Mutex<Option<RecommendedWatcher>>>,
}

#[tokio::main]
async fn main() -> ! {
    let state = AppState {
        connection: Arc::new(Mutex::new(ConnectionState::init())),
        watcher: Arc::new(Mutex::new(None)),
    };

    watch_connection(state.connection.clone(), state.watcher.clone());

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
