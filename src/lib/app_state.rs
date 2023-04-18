use std::{
    io::{self},
    path::{Path, PathBuf},
};

use base64::Engine;
use notify::Event;
use reqwest::Url;

#[derive(Debug)]
pub struct AppState {
    pub league_folder: PathBuf,
    pub lock_file_path: PathBuf,
    pub client_url: Option<ClientConnection>,
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
                    self.client_url = ClientConnection::parse(event_path).ok();
                    true
                }
                Remove(_) => {
                    self.client_url = None;
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

impl ClientConnection {
    pub fn parse<P>(path: P) -> Result<ClientConnection, io::Error>
    where
        P: AsRef<Path>,
    {
        Self::parse_str(std::fs::read_to_string(path)?)
    }

    // TODO:: Cleanup unwraps -- SNAFU?
    pub fn parse_str<S>(s: S) -> Result<ClientConnection, io::Error>
    where
        S: AsRef<str>,
    {
        let s = s.as_ref();
        println!("{}", s);

        let mut lock_file_data = s.split(":");

        if lock_file_data.clone().count() == 5 {
            // Name and PID are junk for now
            let _ = lock_file_data.next(); // Process Name
            let _ = lock_file_data.next(); // Process PID

            let port = lock_file_data.next().unwrap();
            let pass = lock_file_data.next().unwrap().to_owned();
            let protocol = lock_file_data.next().unwrap();

            let auth_token =
                base64::engine::general_purpose::STANDARD.encode(&format!("riot:{pass}"));

            Ok(ClientConnection {
                url: Url::parse(&format!("{protocol}://127.0.0.1:{port}/")).unwrap(),
                pass,
                auth_token,
            })
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Invalid Lockfile"))
        }
    }
}
