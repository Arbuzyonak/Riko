use discord_rich_presence::activity::{Activity, Timestamps};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const DISCORD_CLIENT_ID: &str = "1394086479871213590";
const RECONNECT_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Clone, Debug)]
pub enum PresenceCmd {
    Playing { game_name: String, started_at_unix: i64 },
    Idle,
    Shutdown,
}

#[derive(Clone)]
pub struct PresenceHandle {
    tx: mpsc::Sender<PresenceCmd>,
}

impl PresenceHandle {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || run(rx));
        Self { tx }
    }

    pub fn send(&self, cmd: PresenceCmd) {
        self.tx.send(cmd).ok();
    }
}

struct Connection {
    client: Option<DiscordIpcClient>,
    last_attempt: Option<Instant>,
}

impl Connection {
    fn ensure(&mut self) -> Option<&mut DiscordIpcClient> {
        if self.client.is_some() {
            return self.client.as_mut();
        }
        if let Some(at) = self.last_attempt
            && at.elapsed() < RECONNECT_INTERVAL
        {
            return None;
        }
        self.last_attempt = Some(Instant::now());
        let mut client = DiscordIpcClient::new(DISCORD_CLIENT_ID).ok()?;
        if client.connect().is_err() {
            return None;
        }
        self.client = Some(client);
        self.client.as_mut()
    }

    fn drop_client(&mut self) {
        if let Some(mut client) = self.client.take() {
            client.close().ok();
        }
    }
}

fn run(rx: mpsc::Receiver<PresenceCmd>) {
    let mut conn = Connection {
        client: None,
        last_attempt: None,
    };
    let mut current: Option<PresenceCmd> = None;

    loop {
        let cmd = match rx.recv_timeout(RECONNECT_INTERVAL) {
            Ok(cmd) => Some(cmd),
            Err(mpsc::RecvTimeoutError::Timeout) => None,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };

        match cmd {
            Some(PresenceCmd::Shutdown) => break,
            Some(other) => current = Some(other),
            None => {}
        }

        let ok = match &current {
            Some(PresenceCmd::Playing {
                game_name,
                started_at_unix,
            }) => conn
                .ensure()
                .map(|client| {
                    client
                        .set_activity(
                            Activity::new()
                                .details(game_name)
                                .state("via Riko Launcher")
                                .timestamps(Timestamps::new().start(*started_at_unix)),
                        )
                        .is_ok()
                })
                .unwrap_or(true),
            Some(PresenceCmd::Idle) => conn
                .client
                .as_mut()
                .map(|client| client.clear_activity().is_ok())
                .unwrap_or(true),
            _ => true,
        };

        if !ok {
            conn.drop_client();
        }
    }

    conn.drop_client();
}
