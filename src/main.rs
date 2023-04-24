use std::sync::{Arc, Mutex};

use notify::ReadDirectoryChangesWatcher;
use valor_lib::{watch_connection, ConnectionState};

struct AppState {
    connection: Arc<Mutex<ConnectionState>>,
    watcher: Arc<Mutex<Option<ReadDirectoryChangesWatcher>>>,
}

fn main() -> ! {
    let state = AppState {
        connection: Arc::new(Mutex::new(ConnectionState::init())),
        watcher: Arc::new(Mutex::new(None)),
    };

    {
        let mut watcher = state.watcher.lock().unwrap();
        *watcher = watch_connection(state.connection.clone()).ok();
    }

    loop {}
}
