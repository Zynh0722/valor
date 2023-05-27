use std::{println, sync::Arc};

use crate::{watch_connection, ConnectionState};
use notify::RecommendedWatcher;
use tokio::sync::{oneshot, watch};

use std::sync::Mutex as StdMutex;
use tokio::sync::Mutex as TokioMutex;

#[derive(Debug)]
pub struct Valor {
    #[allow(dead_code)]
    connection: Arc<TokioMutex<ConnectionState>>,
    #[allow(dead_code)]
    watcher: Arc<StdMutex<Option<RecommendedWatcher>>>,
}

impl Valor {
    pub async fn new() -> Self {
        let connection = Arc::new(TokioMutex::new(ConnectionState::none()));
        let watcher = Arc::new(StdMutex::new(None));

        let (watch_tx, mut watch_rx) = watch::channel::<Option<notify::Event>>(None);
        let (init_tx, init_rx) = oneshot::channel::<ConnectionState>();

        let _file_system_watcher = watch_connection(init_tx, watch_tx, watcher.clone());

        // Main app layer start
        {
            let connection = connection.clone();
            tokio::spawn(async move {
                let initial_state = init_rx.await.unwrap();
                let mut connection = connection.lock().await;
                *connection = initial_state;
                print_connection(&connection);
            });
        }

        {
            let connection = connection.clone();
            tokio::spawn(async move {
                while watch_rx.changed().await.is_ok() {
                    let event = watch_rx.borrow().clone().unwrap();

                    let mut connection = connection.lock().await;
                    let updated = connection.update_state(event).await;
                    if updated {
                        print_connection(&connection);
                    }
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
            ui.label("Valor or something idk");
        });
    }
}

fn print_connection(connection: &ConnectionState) {
    println!(
        "lockfile: {:#?}\nknown_path: {:#?}",
        connection.lockfile, connection.known_path
    );
}
