use std::sync::{Arc, Mutex};

use notify::{Event, ReadDirectoryChangesWatcher, RecursiveMode, Watcher};

use crate::ConnectionState;

pub fn watch_connection(
    connection: Arc<Mutex<ConnectionState>>,
) -> notify::Result<ReadDirectoryChangesWatcher> {
    let mut watcher = {
        let connection = connection.clone();
        notify::recommended_watcher(move |res: notify::Result<Event>| match res {
            Ok(event) => {
                let mut connection = connection.lock().unwrap();
                if connection.update_state(event) {
                    println!("{connection:#?}");
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        })?
    };

    {
        let connection = connection.lock().unwrap();

        if let Some(lockfile) = connection.lockfile.as_ref() {
            let league_folder = lockfile.path.parent().unwrap();
            println!("{league_folder:?}");
            watcher.watch(&league_folder, RecursiveMode::NonRecursive)?;
            println!("{connection:#?}");
        }
    }

    Ok(watcher)
}
