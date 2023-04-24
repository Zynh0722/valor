use std::path::Path;

use league_client_connector::{LeagueClientConnector, RiotLockFile};
use notify::Event;
use reqwest::Url;

#[derive(Debug)]
pub struct ConnectionState {
    pub lockfile: Option<RiotLockFile>,
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
        ConnectionState { lockfile }
    }

    pub fn update_state(&mut self, event: Event) -> bool {
        // TODO: figure out if unwrap is ok here, or fix it anyway
        let event_path = event.paths.iter().next().unwrap();

        if self.check_path(event_path) {
            use notify::EventKind::{Create, Modify, Remove};
            return match event.kind {
                // TODO:: Test if Create can be removed on all platforms
                Create(_) | Modify(_) => {
                    self.lockfile = LeagueClientConnector::parse_lockfile().ok();
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
        if let Some(lockfile) = self.lockfile.as_ref() {
            return lockfile.path == path.as_ref();
        };

        false
    }
}
