use std::{
    env,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use base64::Engine;
use dotenv::dotenv;
use notify::{Event, RecursiveMode, Watcher};
use valor_lib::{AppState, ClientState};

fn main() -> notify::Result<()> {
    let state = Arc::new(Mutex::new(init_state()));

    let mut watcher = {
        let state = state.clone();
        // Automatically select the best implementation for your platform.
        notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
            Ok(event) => {
                let mut state = state.lock().unwrap();
                if state.update_state(event) {
                    println!("{:?}", state);
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        })?
    };

    {
        let state = state.lock().unwrap();

        watcher.watch(&state.league_folder, RecursiveMode::NonRecursive)?;

        println!("{:?}", state);
    }

    loop {}
}

fn init_state() -> AppState {
    dotenv().ok();

    let league_folder = env::var("LEAGUE_FOLDER").unwrap();
    let league_folder = Path::new(&league_folder).to_owned();

    let lock_file_path = league_folder.join("lockfile");

    let client_url = ClientState::parse(&lock_file_path);

    AppState {
        league_folder,
        lock_file_path,
        client_url,
    }
}
