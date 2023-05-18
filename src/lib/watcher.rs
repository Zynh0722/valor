use std::time::Duration;

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::{mpsc, oneshot};

use crate::ConnectionState;

pub async fn watch_connection(
    init_tx: oneshot::Sender<ConnectionState>,
    event_tx: mpsc::UnboundedSender<Event>,
) -> RecommendedWatcher {
    {
        let mut connection = ConnectionState::init().await;

        while connection.known_path.is_none() {
            std::thread::sleep(Duration::from_secs(1));
            connection = ConnectionState::init().await;
        }

        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<Event>| match res {
                Ok(event) => event_tx.send(event).unwrap(),
                Err(e) => println!("watch error: {e:?}"),
            })
            .unwrap();

        if let Some(lockfile) = connection.lockfile.as_ref() {
            let league_folder = lockfile.path.parent().unwrap();
            println!("{league_folder:?}");
            watcher
                .watch(&league_folder, RecursiveMode::NonRecursive)
                .unwrap();
            println!("{connection:#?}");
        }

        init_tx.send(connection).unwrap();

        watcher
    }

    // Listen to watcher events with an async task
}
