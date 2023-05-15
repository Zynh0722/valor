use std::{sync::Arc, time::Duration};

use notify::RecommendedWatcher;
use valor_lib::{watch_connection, ConnectionState};

#[derive(Debug)]
struct AppState {
    connection: Arc<tokio::sync::Mutex<ConnectionState>>,
    watcher: Arc<std::sync::Mutex<Option<RecommendedWatcher>>>,
}

#[tokio::main]
async fn main() -> ! {
    let state = AppState {
        connection: Arc::new(tokio::sync::Mutex::new(ConnectionState::init().await)),
        watcher: Arc::new(std::sync::Mutex::new(None)),
    };

    watch_connection(state.connection.clone(), state.watcher.clone()).await;

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
