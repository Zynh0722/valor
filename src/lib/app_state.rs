use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use base64::Engine;
use notify::Event;
use reqwest::Url;

#[derive(Debug)]
pub struct AppState {
    pub league_folder: PathBuf,
    pub lock_file_path: PathBuf,
    pub client_url: ClientState,
}

#[derive(Debug)]
pub enum ClientState {
    Dead,
    Alive(ClientConnection),
}

#[derive(Debug)]
pub struct ClientConnection {
    pub url: Url,
    pub pass: String,
    // This exists pretty much exclusively for debug // TODO:: remove or make feature
    pub auth_token: String,
}

impl AppState {
    pub fn update_state(&mut self, event: Event) -> bool {
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

    pub fn check_path<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        self.lock_file_path == path.as_ref()
    }
}

impl ClientState {
    pub fn parse<P>(path: P) -> ClientState
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

                    let auth_token = base64::engine::general_purpose::STANDARD
                        .encode(&format!("riot:{}", &pass));

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
