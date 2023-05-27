use std::{println, sync::Arc};

use tokio::sync::{oneshot, watch};
use valor_lib::{watch_connection, ConnectionState};

use tokio::sync::Mutex as TokioMutex;

#[derive(Debug)]
struct AppState {
    connection: Arc<TokioMutex<ConnectionState>>,
}

#[derive(Debug)]
struct Valor {}

impl Default for Valor {
    fn default() -> Self {
        Self {}
    }
}

impl eframe::App for Valor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Valor or something idk");
        });
    }
}

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Unable to start tokio runtime");

    // This lets tokio::spawn work
    let _enter = rt.enter();

    std::thread::spawn(move || rt.block_on(tokio_main()));

    eframe::run_native(
        "Valor",
        eframe::NativeOptions::default(),
        Box::new(move |_cc| Box::new(Valor::default())),
    )
    .expect("Failed to start graphics context");
}

// TODO: I believe all of this needs to be moved into the Valor eframe app
async fn tokio_main() {
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
            print_connection(&connection);
        });
    }

    let watch_handle = tokio::spawn(async move {
        while watch_rx.changed().await.is_ok() {
            let event = watch_rx.borrow().clone().unwrap();

            let mut connection = state.connection.lock().await;
            let updated = connection.update_state(event).await;
            if updated {
                print_connection(&connection);
            }
        }
    });

    tokio::select! {
        _ = watch_handle => (),
    }
    // Main app layer end
}

fn print_connection(connection: &ConnectionState) {
    println!(
        "lockfile: {:#?}\nknown_path: {:#?}",
        connection.lockfile, connection.known_path
    );
}
