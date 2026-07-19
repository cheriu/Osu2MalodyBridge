use axum::http::HeaderMap;
use base64::Engine;
use reqwest::Client as HttpClient;
use rosu_v2::Osu;
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::RsaPublicKey;
use sha2::{Digest, Sha256};
use tracing::{error, warn};

use crate::cache::{BeatmapCache, ListSearchCache, PromoteSearchCache};
use crate::config::Config;
use crate::models::*;

mod charts;
mod download;
mod search;
pub mod test_utils;

/// Malody RSA public key for verifying client `key` signatures (PKCS#1, base64-encoded DER).
const MALODY_RSA_PUBKEY: &str = "MIGJAoGBALIEG+2iLpMPDqsqr3onEUHu+QtdRANEC6TBl7UtEAm1WondcyAiXsiIYmAZgptOcoLBZH6IfqmoXsv/DbVOdTbyQcgRntwqYEk4mh/boQHbCyOC7bVaoVGEAyaSNDW0WeBaUzAHfa3D8Fe6eDdyvKB2imhS2338ndYvT5dNNUV5AgMBAAE=";

pub struct AppState {
    pub config: Config,
    osu: Option<Osu>,
    pub http_client: HttpClient,
    pub beatmap_cache: BeatmapCache,
    pub list_cache: ListSearchCache,
    pub promote_cache: PromoteSearchCache,
    /// Parsed Malody RSA public key for uid/key verification.
    malody_pubkey: Option<RsaPublicKey>,
}

impl AppState {
    pub fn new(config: Config, osu: Osu) -> Self {
        let http_client = HttpClient::builder()
            .user_agent("osu2malody-store/0.1.0 (+https://github.com/cheriu/Osu2MalodyBridge)")
            .build()
            .expect("Failed to build HTTP client");

        let malody_pubkey = parse_malody_pubkey();

        Self {
            config,
            osu: Some(osu),
            http_client,
            beatmap_cache: BeatmapCache::new(),
            list_cache: ListSearchCache::new(),
            promote_cache: PromoteSearchCache::new(),
            malody_pubkey,
        }
    }

    /// Get a reference to the osu! API client. Panics if not initialized (test builds).
    pub(crate) fn osu_client(&self) -> &Osu {
        self.osu
            .as_ref()
            .expect("osu! API client not initialized")
    }

    /// Returns the osu! client if available; None in test builds without credentials.
    pub(crate) fn osu_opt(&self) -> Option<&Osu> {
        self.osu.as_ref()
    }

    /// Verify Malody client auth params (uid, key, api) when `verify_client_auth` is enabled.
    /// When enabled, validates that the `key` is a valid RSA-SHA256 signature of `uid`.
    /// Returns 0 on success, or an error code to use in the response.
    pub fn verify_client_auth(&self, uid: Option<i32>, key: Option<&str>, api: Option<i32>) -> i32 {
        if !self.config.malody.server.verify_client_auth {
            return 0;
        }
        let uid_val = match uid {
            Some(v) => v,
            None => {
                warn!("Client auth: missing uid");
                return -1;
            }
        };
        let key_str = match key {
            Some(v) => v,
            None => {
                warn!("Client auth: missing key");
                return -1;
            }
        };
        let api_val = match api {
            Some(v) => v,
            None => {
                warn!("Client auth: missing api");
                return -1;
            }
        };
        if api_val < self.config.malody.server.min {
            warn!(
                "Client auth: api version {} below minimum {}",
                api_val,
                self.config.malody.server.min
            );
            return -1;
        }

        // RSA signature verification of uid with the key parameter
        match &self.malody_pubkey {
            Some(pubkey) => {
                if !verify_uid_signature(pubkey, uid_val, key_str) {
                    warn!("Client auth: invalid key signature for uid={}", uid_val);
                    return -1;
                }
            }
            None => {
                // Public key failed to parse — treat any key as valid (graceful fallback)
                warn!("Client auth: Malody RSA public key not available, skipping signature check");
            }
        }
        0
    }
}

/// Parse the hardcoded Malody RSA public key (PKCS#1 DER, base64-encoded).
fn parse_malody_pubkey() -> Option<RsaPublicKey> {
    let der = base64::engine::general_purpose::STANDARD
        .decode(MALODY_RSA_PUBKEY)
        .ok()?;
    RsaPublicKey::from_pkcs1_der(&der).ok()
}

/// Verify that `key` is a valid RSA-SHA256 (PKCS1v1.5) signature of `uid`.
fn verify_uid_signature(pubkey: &RsaPublicKey, uid: i32, key: &str) -> bool {
    let uid_str = uid.to_string();
    let digest = Sha256::digest(uid_str.as_bytes());

    // Malody uses URL-safe base64 for the signature ( - → + , _ → / ).
    // URL query params often drop trailing = padding, so try both.
    let key_standard = key.replace('-', "+").replace('_', "/");
    let sig_bytes = base64::engine::general_purpose::STANDARD
        .decode(&key_standard)
        .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(&key_standard));

    let sig_bytes = match sig_bytes {
        Ok(b) => b,
        Err(_) => return false,
    };

    pubkey
        .verify(
            rsa::Pkcs1v15Sign::new::<Sha256>(),
            &digest,
            &sig_bytes,
        )
        .is_ok()
}

const API_BASE_PATH: &str = "api/store";
const DOWNLOAD_BASE: &str = "https://mirror.hinamizawa.ai/api/v1/hinai/d";

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn server_info(state: &AppState) -> ServerInfoResponse {
    ServerInfoResponse {
        code: 0,
        api: state.config.malody.server.api,
        min: state.config.malody.server.min,
        welcome: state.config.malody.server.welcome.clone(),
    }
}

pub async fn song_list(state: &AppState, params: &ListQueryParams) -> PagedResponse<Song> {
    let mode = params.mode.unwrap_or(-1);
    match MalodyMode::from_i32(mode) {
        Some(MalodyMode::Any) | Some(MalodyMode::Key) => {}
        _ => {
            return PagedResponse {
                code: -1,
                has_more: false,
                next: 0,
                data: vec![],
            };
        }
    }
    match search::do_song_list(state, params).await {
        Ok(resp) => resp,
        Err(e) => {
            error!("List error for {:?}: {:?}", params, e);
            PagedResponse {
                code: -1,
                has_more: false,
                next: 0,
                data: vec![],
            }
        }
    }
}

pub async fn song_promote(state: &AppState, params: &PromoteQueryParams) -> PagedResponse<Song> {
    match search::do_song_promote(state, params).await {
        Ok(resp) => resp,
        Err(e) => {
            error!("Promote error for {:?}: {:?}", params, e);
            PagedResponse {
                code: -1,
                has_more: false,
                next: 0,
                data: vec![],
            }
        }
    }
}

pub async fn charts_list(state: &AppState, sid: i32) -> PagedResponse<Chart> {
    match charts::do_charts_list(state, sid).await {
        Ok(resp) => resp,
        Err(e) => {
            error!("Charts error for sid={}: {:?}", sid, e);
            PagedResponse {
                code: -1,
                has_more: false,
                next: 0,
                data: vec![],
            }
        }
    }
}

pub async fn download_chart(
    state: &AppState,
    cid: i32,
    headers: &HeaderMap,
) -> DownloadResponse {
    let beatmap_entry = match download::get_beatmap_entry(state, cid as u32).await {
        Ok(entry) => entry,
        Err(e) => {
            error!("Download: beatmap not found for cid={}: {:?}", cid, e);
            return DownloadResponse {
                code: -2,
                items: vec![],
                sid: 0,
                cid,
            };
        }
    };

    let mapset_id = beatmap_entry.mapset_id;

    match download::do_download_with_entry(state, headers, &beatmap_entry).await {
        Ok(items) => DownloadResponse {
            code: 0,
            items,
            sid: mapset_id as i32,
            cid,
        },
        Err(e) => {
            error!("Download error for cid={}: {:?}", cid, e);
            DownloadResponse {
                code: -2,
                items: vec![],
                sid: mapset_id as i32,
                cid,
            }
        }
    }
}

pub async fn send_resource(
    state: &AppState,
    cid: i32,
    resource_type: &str,
) -> Result<(Vec<u8>, String), (u16, String)> {
    download::do_send_resource(state, cid, resource_type).await
}
