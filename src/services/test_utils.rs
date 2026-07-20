/// Utilities for tests — creates AppState without needing real osu! credentials.
use reqwest::Client as HttpClient;

use crate::cache::{BeatmapCache, SearchChainCache};
use crate::config::Config;

use super::AppState;

/// Build a minimal AppState for tests with no osu! client.
/// Endpoints that need the osu! client will return errors.
pub fn dummy_state() -> AppState {
    let config_yaml = r#"
server:
  port: 0
malody:
  server:
    api: 202310
    min: 202310
    welcome: "test server"
    tmp: "/tmp/malody-test"
    verify_client_auth: false
  osu:
    clientID: 0
    clientSecret: ""
"#;
    let config: Config = serde_yaml::from_str(config_yaml).expect("test config parse");
    dummy_state_with_config(config)
}

pub fn dummy_state_with_config(config: Config) -> AppState {
    AppState {
        config,
        osu: None,
        http_client: HttpClient::new(),
        beatmap_cache: BeatmapCache::new(),
        list_chain: SearchChainCache::new(),
        promote_chain: SearchChainCache::new(),
        malody_pubkey: None,
    }
}

/// Build an AppState with a live osu! API client.
/// Set `OSU_CLIENT_ID` and `OSU_CLIENT_SECRET` env vars (matching server config).
/// Returns `None` if credentials are not set or client creation fails.
pub async fn live_state() -> Option<AppState> {
    // rustls 0.23: install crypto provider before Osu client uses HTTPS.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let client_id: u64 = std::env::var("OSU_CLIENT_ID").ok()?.parse().ok()?;
    let client_secret = std::env::var("OSU_CLIENT_SECRET").ok()?;

    let osu = rosu_v2::Osu::builder()
        .client_id(client_id)
        .client_secret(client_secret)
        .ratelimit(5)
        .build()
        .await
        .ok()?;

    let config_yaml = r#"
server:
  port: 0
malody:
  server:
    api: 202310
    min: 202310
    welcome: "test server (live)"
    tmp: "/tmp/malody-test"
    verify_client_auth: false
  osu:
    clientID: 0
    clientSecret: ""
"#;
    let config: Config = serde_yaml::from_str(config_yaml).expect("test config parse");

    Some(AppState {
        config,
        osu: Some(osu),
        http_client: HttpClient::new(),
        beatmap_cache: BeatmapCache::new(),
        list_chain: SearchChainCache::new(),
        promote_chain: SearchChainCache::new(),
        malody_pubkey: None,
    })
}
