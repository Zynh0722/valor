use std::{sync::Arc, time::Duration};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

use crate::ConnectionState;

pub async fn watch_connection(
    connection: Arc<tokio::sync::Mutex<ConnectionState>>,
    watcher: Arc<std::sync::Mutex<Option<RecommendedWatcher>>>,
) {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Event>();

    {
        let connection = connection.clone();

        if connection.lock().await.known_path.is_none() {
            let mut inner_connection = ConnectionState::init().await;

            while inner_connection.known_path.is_none() {
                std::thread::sleep(Duration::from_secs(1));
                inner_connection = ConnectionState::init().await;
            }

            let mut connection = connection.lock().await;
            *connection = inner_connection;
        }

        let outer_watcher = watcher;
        // Create the watcher struct
        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<Event>| match res {
                Ok(event) => tx.send(event).unwrap(),
                Err(e) => println!("watch error: {e:?}"),
            })
            .unwrap();

        {
            let connection = connection.lock().await;

            if let Some(lockfile) = connection.lockfile.as_ref() {
                let league_folder = lockfile.path.parent().unwrap();
                println!("{league_folder:?}");
                watcher
                    .watch(&league_folder, RecursiveMode::NonRecursive)
                    .unwrap();
                println!("{connection:#?}");
            }
        }

        // Not exactly sure if this means the watching will be done in the blocking thread,
        // honestly its probably fine if it doesn't, either way Im leaving this how it is
        // for now
        {
            let mut outer_watcher = outer_watcher.lock().unwrap();

            *outer_watcher = Some(watcher);
        }
    }

    // Listen to watcher events with an async task
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let mut connection = connection.lock().await;
            if connection.update_state(event).await {
                println!("{connection:#?}");
            }
        }
    });
}
