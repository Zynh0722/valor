use std::path::{Path, PathBuf};

use league_client_connector::{LeagueClientConnector, RiotLockFile};
use native_tls::TlsConnector;
use notify::Event;
use reqwest::Url;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::{
        handshake::client::generate_key,
        http::{Request, Uri},
    },
    Connector, MaybeTlsStream, WebSocketStream,
};

#[derive(Debug)]
pub struct ConnectionState {
    pub lockfile: Option<RiotLockFile>,
    pub known_path: Option<PathBuf>,
    pub ws_stream: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

#[derive(Debug)]
pub struct ClientConnection {
    pub url: Url,
    pub pass: String,
    // This exists pretty much exclusively for debug // TODO: remove or make feature
    pub auth_token: String,
}

impl ConnectionState {
    pub async fn init() -> Self {
        let lockfile = LeagueClientConnector::parse_lockfile().ok();
        let known_path = lockfile.as_ref().map(|lf| lf.path.clone());

        let ws_stream = if let Some(lockfile) = lockfile.as_ref() {
            Self::get_ws_stream(lockfile)
                .await
                .map_err(|err| {
                    println!("{err:?}");
                    err
                })
                .ok()
        } else {
            None
        };

        ConnectionState {
            lockfile,
            known_path,
            ws_stream,
        }
    }

    pub async fn update_state(&mut self, event: Event) -> bool {
        // TODO: figure out if unwrap is ok here, or fix it anyway
        let event_path = event.paths.iter().next().unwrap();

        if self.check_path(event_path) {
            use notify::EventKind::{Modify, Remove};

            return match event.kind {
                // TODO: Test if Create can be removed on all platforms
                // ! I think it can, as the LCU first creates an empty
                //   lockfile, then fills it with data, but this behaviour
                //   may change depending on platform
                Modify(_) => {
                    let mut changed = false;

                    if let Some(known_path) = self.known_path.as_ref() {
                        self.lockfile =
                            LeagueClientConnector::parse_lockfile_from_path(known_path).ok();
                        changed = true;
                    }

                    if let Some(lockfile) = self.lockfile.as_ref() {
                        self.ws_stream = Self::get_ws_stream(lockfile).await.ok();
                        changed = true;
                    }

                    changed
                }
                Remove(_) => {
                    self.lockfile = None;
                    self.ws_stream = None;
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

    async fn get_ws_stream(
        lockfile: &RiotLockFile,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, tokio_tungstenite::tungstenite::Error>
    {
        // I'd like to offload some of this manual request building to the tungstenite crate,
        // as this is basically copy paste with an additional header
        let uri: Uri = format!("wss://{}:{}/", lockfile.address, lockfile.port)
            .parse()
            .unwrap();

        let authority = uri.authority().unwrap().as_str();
        let host = authority
            .find('@')
            .map(|idx| authority.split_at(idx + 1).1)
            .unwrap_or_else(|| authority);

        let ws_path = Request::builder()
            .method("GET")
            .header("Host", host)
            .header("Authorization", &format!("Basic {}", lockfile.b64_auth))
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", generate_key())
            .uri(uri)
            .body(())
            .unwrap();

        println!("{ws_path:?}");

        let danger_connector = TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        let danger_connector = Connector::NativeTls(danger_connector);

        connect_async_tls_with_config(ws_path, None, Some(danger_connector))
            .await
            .map(|(stream, _)| stream)
    }
}
