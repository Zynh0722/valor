use crate::Valor;

use crate::{watch_connection, ConnectionState};
use std::sync::{Arc, Mutex};
use tokio::sync::{oneshot, watch};

impl Valor {
    pub async fn new() -> Self {
        let connection = Arc::new(Mutex::new(ConnectionState::none()));

        let (watch_tx, mut watch_rx) = watch::channel::<Option<notify::Event>>(None);
        let (init_tx, init_rx) = oneshot::channel::<ConnectionState>();

        let _file_system_watcher = watch_connection(init_tx, watch_tx);

        // Main app layer start
        {
            let connection = connection.clone();
            tokio::spawn(async move {
                let initial_state = init_rx.await.unwrap();
                let mut connection = connection.lock().unwrap();
                *connection = initial_state;
            });
        }

        {
            let connection = connection.clone();
            tokio::spawn(async move {
                while watch_rx.changed().await.is_ok() {
                    let event = watch_rx.borrow().clone().unwrap();

                    let mut connection = connection.lock().unwrap();
                    connection.update_state(event);
                }
            });
        }

        // Main app layer end
        Self { connection }
    }
}
