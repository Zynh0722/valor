use std::path::{Path, PathBuf};

use league_client_connector::{LeagueClientConnector, RiotLockFile};
use notify::Event;
use reqwest::Url;

#[derive(Debug)]
pub struct ConnectionState {
    pub lockfile: Option<RiotLockFile>,
    pub known_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct ClientConnection {
    pub url: Url,
    pub pass: String,
    // This exists pretty much exclusively for debug // TODO:: remove or make feature
    pub auth_token: String,
}

impl ConnectionState {
    pub fn init() -> Self {
        let lockfile = LeagueClientConnector::parse_lockfile().ok();
        let known_path = lockfile.as_ref().map(|lf| lf.path.clone());
        ConnectionState {
            lockfile,
            known_path,
        }
    }

    pub fn update_state(&mut self, event: Event) -> bool {
        // TODO: figure out if unwrap is ok here, or fix it anyway
        let event_path = event.paths.iter().next().unwrap();

        if self.check_path(event_path) {
            use notify::EventKind::{Modify, Remove};

            return match event.kind {
                // TODO:: Test if Create can be removed on all platforms
                // ! I think it can, as the LCU first creates an empty
                //   lockfile, then fills it with data, but this behaviour
                //   may change depending on platform
                Modify(_) => {
                    self.lockfile = LeagueClientConnector::parse_lockfile_from_path(
                        self.known_path.as_ref().unwrap(),
                    )
                    .ok();
                    true
                }
                Remove(_) => {
                    self.lockfile = None;
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
        if let Some(known_path) = self.known_path.as_ref() {
            return known_path == path.as_ref();
        };

        false
    }
}
