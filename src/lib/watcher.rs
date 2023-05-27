use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::{oneshot, watch};

use crate::ConnectionState;

pub fn watch_connection(
    init_tx: oneshot::Sender<ConnectionState>,
    event_tx: watch::Sender<Option<Event>>,
    watcher: Arc<Mutex<Option<RecommendedWatcher>>>,
) {
    tokio::spawn(async move {
        let mut connection = ConnectionState::init().await;

        while connection.known_path.is_none() {
            tokio::time::sleep(Duration::from_secs(1)).await;
            connection = ConnectionState::init().await;
        }

        let mut inner_watcher = notify::recommended_watcher(move |res| match res {
            Ok(event) => send_event(&event_tx, event),
            Err(e) => println!("watch error: {e:?}"),
        })
        .unwrap();

        // Unwrap is safe here because known_path is Some
        let lockfile = connection.lockfile.as_ref().unwrap();
        let league_folder = lockfile.path.parent().unwrap();
        inner_watcher
            .watch(&league_folder, RecursiveMode::NonRecursive)
            .unwrap();

        init_tx.send(connection).unwrap();

        let mut watcher = watcher.lock().unwrap();
        *watcher = Some(inner_watcher);
    });
}

fn send_event(tx: &watch::Sender<Option<Event>>, event: Event) {
    if is_lockfile_event(&event) {
        tx.send(Some(event)).unwrap();
    }
}

fn is_lockfile_event(event: &Event) -> bool {
    event.paths.iter().next().map_or(false, is_lockfile_path)
}

fn is_lockfile_path(path: &PathBuf) -> bool {
    path.file_name().map_or(false, |name| name == "lockfile")
}
