use std::sync::{Arc, Mutex};

use notify::{RecursiveMode, Watcher, ReadDirectoryChangesWatcher, Event};

use crate::ConnectionState;

pub fn watch_connection(connection: Arc<Mutex<ConnectionState>>) -> notify::Result<ReadDirectoryChangesWatcher> {
    let mut watcher = {
        let connection = connection.clone();
        notify::recommended_watcher(move |res: notify::Result<Event>| match res {
            Ok(event) => {
                let mut connection = connection.lock().unwrap();
                if connection.update_state(event) {
                    println!("{:?}", connection);
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        })?
    };

    {
        let connection = connection.lock().unwrap();
        watcher.watch(&connection.league_folder, RecursiveMode::NonRecursive)?;
        println!("{connection:?}");
    }

    Ok(watcher)
}
