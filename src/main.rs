use std::sync::Arc;

use tokio::sync::mpsc;
use valor_lib::{watch_connection, ConnectionState};

use tokio::sync::Mutex as TokioMutex;

#[derive(Debug)]
struct AppState {
    connection: Arc<TokioMutex<ConnectionState>>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        connection: Arc::new(TokioMutex::new(ConnectionState::init().await)),
    };

    let (tx, mut rx) = mpsc::unbounded_channel::<notify::Event>();

    let _file_system_watcher = watch_connection(state.connection.clone(), tx.clone()).await;

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let mut connection = state.connection.lock().await;
            if connection.update_state(event).await {
                println!("{connection:#?}");
            }
        }
    })
    .await
    .unwrap();
}
