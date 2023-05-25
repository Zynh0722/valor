use std::{println, sync::Arc};

use tokio::sync::{oneshot, watch};
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

    let (watch_tx, mut watch_rx) = watch::channel::<Option<notify::Event>>(None);
    let (init_tx, init_rx) = oneshot::channel::<ConnectionState>();

    let _file_system_watcher = watch_connection(init_tx, watch_tx).await;

    // Main app layer start
    {
        let connection = state.connection.clone();
        tokio::spawn(async move {
            let initial_state = init_rx.await.unwrap();
            let mut connection = connection.lock().await;
            *connection = initial_state;
            println!(
                "lockfile: {:#?}\nknown_path: {:#?}",
                connection.lockfile, connection.known_path
            );
        });
    }

    let watch_handle = tokio::spawn(async move {
        while watch_rx.changed().await.is_ok() {
            let event = watch_rx.borrow().clone().unwrap();

            let mut connection = state.connection.lock().await;
            let updated = connection.update_state(event).await;
            if updated {
                println!(
                    "lockfile: {:#?}\nknown_path: {:#?}",
                    connection.lockfile, connection.known_path
                );
            }
        }
    });

    tokio::select! {
        _ = watch_handle => (),
    }
    // Main app layer end
}
