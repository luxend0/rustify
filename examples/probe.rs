//! W0 probe: OAuth login, session connect, play one track, report whether the
//! account still gets audio keys (librespot fails for accounts after ~2024/25).
//! Usage: cargo run --release --example probe [spotify:track:...]
use std::time::Duration;

use librespot_core::{SessionConfig, Session, SpotifyUri, authentication::Credentials, cache::Cache};
use librespot_oauth::OAuthClientBuilder;
use librespot_playback::{
    audio_backend, config::{AudioFormat, PlayerConfig}, mixer::NoOpVolume, player::{Player, PlayerEvent},
};

const KEYMASTER: &str = "65b708073fc0480ea92a077233ca87bd";
const REDIRECT: &str = "http://127.0.0.1:8898/login";

#[tokio::main]
async fn main() {
    env_logger::Builder::new().parse_filters(&std::env::var("PROBE_LOG").unwrap_or("warn,librespot=info".into())).init();
    let uri = std::env::args().nth(1).unwrap_or("spotify:track:4cOdK2wGLETKBW3PvgPWqT".into());
    let dir = dirs_cache().join("rustify");
    let cache = Cache::new(Some(dir.clone()), None, Some(dir.join("audio")), Some(5 << 30)).unwrap();

    let creds = match cache.credentials() {
        Some(c) => c,
        None => {
            let tok = OAuthClientBuilder::new(KEYMASTER, REDIRECT, vec!["streaming"])
                .open_in_browser()
                .with_custom_message("<h1>Rustify: login ok, back to the terminal.</h1>")
                .build().unwrap()
                .get_access_token_async().await.expect("oauth failed");
            Credentials::with_access_token(tok.access_token)
        }
    };

    let session = Session::new(SessionConfig::default(), Some(cache));
    session.connect(creds, true).await.expect("session connect failed");
    println!("session ok, user {}", session.username());

    let sink = audio_backend::find(Some("pulseaudio".into())).expect("no pulseaudio backend");
    let player = Player::new(PlayerConfig::default(), session, Box::new(NoOpVolume), move || sink(None, AudioFormat::default()));
    let mut events = player.get_player_event_channel();
    player.load(SpotifyUri::from_uri(&uri).expect("bad uri"), true, 0);

    let verdict = tokio::time::timeout(Duration::from_secs(20), async {
        while let Some(ev) = events.recv().await {
            match ev {
                PlayerEvent::Playing { .. } => {
                    tokio::time::sleep(Duration::from_secs(6)).await;
                    return "AUDIO KEY OK - librespot plays for this account";
                }
                PlayerEvent::Unavailable { .. } | PlayerEvent::EndOfTrack { .. } => {
                    return "UNAVAILABLE - no audio key (see log above)";
                }
                _ => {}
            }
        }
        "player channel closed"
    }).await.unwrap_or("TIMEOUT - no playback within 20 s");
    player.stop();
    println!("{verdict}");
}

fn dirs_cache() -> std::path::PathBuf {
    std::env::var_os("XDG_CACHE_HOME").map(Into::into)
        .unwrap_or_else(|| std::path::PathBuf::from(std::env::var_os("HOME").unwrap()).join(".cache"))
}
