use std::sync::Arc;

use crate::{watch_connection, ConnectionState};
use notify::RecommendedWatcher;
use tokio::sync::{oneshot, watch};

use std::sync::Mutex;

#[derive(Debug)]
pub struct Valor {
    #[allow(dead_code)]
    connection: Arc<Mutex<ConnectionState>>,
    #[allow(dead_code)]
    watcher: Arc<Mutex<Option<RecommendedWatcher>>>,
}

impl Valor {
    pub async fn new() -> Self {
        let connection = Arc::new(Mutex::new(ConnectionState::none()));
        let watcher = Arc::new(Mutex::new(None));

        let (watch_tx, mut watch_rx) = watch::channel::<Option<notify::Event>>(None);
        let (init_tx, init_rx) = oneshot::channel::<ConnectionState>();

        let _file_system_watcher = watch_connection(init_tx, watch_tx, watcher.clone());

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
        Self {
            connection,
            watcher,
        }
    }
}

impl eframe::App for Valor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let label = if self.connection.lock().unwrap().lockfile.is_some() {
                "Connected"
            } else {
                "Disconnected"
            };
            ui.label(label);
        });
    }
}
