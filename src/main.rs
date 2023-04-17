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
use reqwest::Url;

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

#[derive(Debug)]
#[allow(dead_code)]
struct AppState {
    league_folder: PathBuf,
    lock_file_path: PathBuf,
    client_url: ClientState,
}

#[derive(Debug)]
enum ClientState {
    Dead,
    Alive(ClientConnection),
}

#[derive(Debug)]
#[allow(dead_code)]
struct ClientConnection {
    url: Url,
    pass: String,
    auth_token: String,
}

impl AppState {
    fn update_state(&mut self, event: Event) -> bool {
        // TODO: figure out if unwrap is ok here, or fix it anyway
        let event_path = event.paths.iter().next().unwrap();

        if self.check_path(event_path) {
            use notify::EventKind::{Create, Modify, Remove};
            return match event.kind {
                // TODO:: Test if Create can be removed on all platforms
                Create(_) | Modify(_) => {
                    self.client_url = ClientState::parse(event_path);
                    true
                }
                Remove(_) => {
                    self.client_url = ClientState::Dead;
                    true
                }
                _ => false,
            };
        }

        false
    }

    fn check_path<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        self.lock_file_path == path.as_ref()
    }
}

impl ClientState {
    fn parse<P>(path: P) -> ClientState
    where
        P: AsRef<Path>,
    {
        match File::open(path).as_mut() {
            Ok(file) => {
                let mut lock_file_data = String::new();
                file.read_to_string(&mut lock_file_data).unwrap();

                println!("{}", &lock_file_data);

                let lock_file_data = lock_file_data.split(":");

                if lock_file_data.clone().count() == 5 {
                    // LeagueClient and PID are junk
                    let mut lock_file_data = lock_file_data.skip(2);

                    let port = lock_file_data.next().unwrap();
                    let pass = lock_file_data.next().unwrap().to_owned();

                    let auth_token = base64::engine::general_purpose::STANDARD.encode(&format!("riot:{}", &pass));

                    ClientState::Alive(ClientConnection {
                        url: Url::parse(&format!("https://127.0.0.1:{}/", port)).unwrap(),
                        pass,
                        auth_token,
                    })
                } else {
                    ClientState::Dead
                }
            }
            Err(_) => ClientState::Dead,
        }
    }
}
