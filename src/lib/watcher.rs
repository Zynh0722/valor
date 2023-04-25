use std::sync::{Arc, Mutex};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::task::spawn_blocking;

use crate::ConnectionState;

pub fn watch_connection(
    connection: Arc<Mutex<ConnectionState>>,
    watcher: Arc<Mutex<Option<RecommendedWatcher>>>,
) {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Event>();

    {
        let connection = connection.clone();
        spawn_blocking(move || {
            let outer_watcher = watcher;
            // Create the watcher struct
            let mut watcher =
                notify::recommended_watcher(move |res: notify::Result<Event>| match res {
                    Ok(event) => tx.send(event).unwrap(),
                    Err(e) => println!("watch error: {e:?}"),
                })
                .unwrap();

            {
                let connection = connection.lock().unwrap();

                if let Some(lockfile) = connection.lockfile.as_ref() {
                    let league_folder = lockfile.path.parent().unwrap();
                    println!("{league_folder:?}");
                    watcher
                        .watch(&league_folder, RecursiveMode::NonRecursive)
                        .unwrap();
                    println!("{connection:#?}");
                }
            }

            {
                let mut outer_watcher = outer_watcher.lock().unwrap();

                *outer_watcher = Some(watcher);
            }
        });
    }
    // Listen to watcher events with an async task
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let mut connection = connection.lock().unwrap();
            if connection.update_state(event) {
                println!("{connection:#?}");
            }
        }
    });
}
