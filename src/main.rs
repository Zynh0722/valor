use std::sync::Arc;

use tokio::sync::{mpsc, oneshot};
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

    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel::<notify::Event>();
    let (init_tx, init_rx) = oneshot::channel::<ConnectionState>();

    let _file_system_watcher = watch_connection(init_tx, notify_tx.clone()).await;

    // Main app layer start
    {
        let connection = state.connection.clone();
        tokio::spawn(async move {
            let initial_state = init_rx.await.unwrap();
            *connection.lock().await = initial_state;
        });
    }

    tokio::spawn(async move {
        while let Some(event) = notify_rx.recv().await {
            let mut connection = state.connection.lock().await;
            if connection.update_state(event).await {
                println!("{connection:#?}");
            }
        }
    })
    .await
    .unwrap();
    // Main app layer end
}
