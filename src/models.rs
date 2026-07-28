use rosu_v2::prelude::GameMode;
use serde::{Deserialize, Deserializer, Serialize};

/// Malody game mode definitions per the Malody Store API spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MalodyMode {
    Any = -1,
    Key = 0,
    Catch = 3,
    Pad = 4,
    Taiko = 5,
    Ring = 6,
    Slide = 7,
    Live = 8,
    Cube = 9,
}

impl MalodyMode {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            -1 => Some(Self::Any),
            0 => Some(Self::Key),
            3 => Some(Self::Catch),
            4 => Some(Self::Pad),
            5 => Some(Self::Taiko),
            6 => Some(Self::Ring),
            7 => Some(Self::Slide),
            8 => Some(Self::Live),
            9 => Some(Self::Cube),
            _ => None,
        }
    }

    /// Whether this mode maps to an osu! game mode we can actually search.
    /// Modes without an osu! equivalent (Pad, Ring, Slide, Live, Cube)
    /// return false — the caller should return empty results, not an error.
    pub fn is_osu_searchable(self) -> bool {
        matches!(self, Self::Any | Self::Key | Self::Catch | Self::Taiko)
    }

    /// Map Malody mode to the osu! GameMode used for API searches.
    /// Only meaningful when `is_osu_searchable()` is true.
    pub fn to_osu_game_mode(self) -> GameMode {
        match self {
            Self::Catch => GameMode::Catch,
            Self::Taiko => GameMode::Taiko,
            // Any, Key, and all Malody-only modes default to Mania
            _ => GameMode::Mania,
        }
    }

    /// Convert an osu! beatmap's GameMode to the corresponding Malody mode.
    pub fn from_osu_game_mode(mode: GameMode) -> Self {
        match mode {
            GameMode::Osu => Self::Key,
            GameMode::Taiko => Self::Taiko,
            GameMode::Catch => Self::Catch,
            GameMode::Mania => Self::Key,
        }
    }
}

impl Default for MalodyMode {
    fn default() -> Self {
        Self::Any
    }
}

/// Custom deserializer for `MalodyMode` from an i32 query parameter.
/// Invalid value → deserialization error → axum returns 400.
fn deserialize_mode<'de, D: Deserializer<'de>>(deserializer: D) -> Result<MalodyMode, D::Error> {
    let value = i32::deserialize(deserializer)?;
    MalodyMode::from_i32(value).ok_or_else(|| {
        serde::de::Error::invalid_value(
            serde::de::Unexpected::Signed(value as i64),
            &"a valid Malody mode (-1, 0, 3, 4, 5, 6, 7, 8, 9)",
        )
    })
}

fn default_mode() -> MalodyMode {
    MalodyMode::Any
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Chart {
    pub cid: i32,
    pub uid: i32,
    pub creator: String,
    pub version: String,
    pub level: i32,
    pub length: i32,
    #[serde(rename = "type")]
    pub chart_type: i32,
    pub size: i32,
    pub mode: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub sid: i32,
    pub cover: String,
    pub length: i32,
    pub bpm: f32,
    pub title: String,
    pub artist: String,
    pub mode: i32,
    pub time: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadItem {
    pub name: String,
    pub hash: String,
    pub file: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadResponse {
    pub code: i32,
    pub items: Vec<DownloadItem>,
    pub sid: i32,
    pub cid: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub code: i32,
    pub has_more: bool,
    pub next: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfoResponse {
    pub code: i32,
    pub api: i32,
    pub min: i32,
    pub welcome: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ListQueryParams {
    #[serde(default)]
    pub word: Option<String>,
    #[serde(default)]
    pub org: Option<i32>,
    #[serde(default = "default_mode", deserialize_with = "deserialize_mode")]
    pub mode: MalodyMode,
    #[serde(default)]
    pub lvge: Option<i32>,
    #[serde(default)]
    pub lvle: Option<i32>,
    #[serde(default)]
    pub beta: Option<i32>,
    #[serde(default)]
    pub from: Option<i32>,
    /// Malody client auth: user ID (global param per spec)
    #[serde(default)]
    pub uid: Option<i32>,
    /// Malody client auth: verification key (global param per spec)
    #[serde(default)]
    pub key: Option<String>,
    /// Malody client auth: client API version (global param per spec)
    #[serde(default)]
    pub api: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PromoteQueryParams {
    #[serde(default)]
    pub org: Option<i32>,
    #[serde(default = "default_mode", deserialize_with = "deserialize_mode")]
    pub mode: MalodyMode,
    #[serde(default)]
    pub from: Option<i32>,
    #[serde(default)]
    pub uid: Option<i32>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub api: Option<i32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ChartsQueryParams {
    pub sid: i32,
    #[serde(default)]
    pub beta: Option<i32>,
    #[serde(default = "default_mode", deserialize_with = "deserialize_mode")]
    pub mode: MalodyMode,
    #[serde(default)]
    pub from: Option<i32>,
    #[serde(default)]
    pub promote: Option<i32>,
    #[serde(default)]
    pub uid: Option<i32>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub api: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct DownloadQueryParams {
    pub cid: i32,
    #[serde(default)]
    pub uid: Option<i32>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub api: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceQueryParams {
    #[serde(rename = "type")]
    pub resource_type: String,
    #[serde(default)]
    pub uid: Option<i32>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub api: Option<i32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct SongQueryParams {
    #[serde(default)]
    pub sid: Option<i32>,
    #[serde(default)]
    pub cid: Option<i32>,
    #[serde(default)]
    pub org: Option<i32>,
    #[serde(default)]
    pub uid: Option<i32>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub api: Option<i32>,
}

impl ListQueryParams {
    pub fn uid(&self) -> Option<i32> { self.uid }
    pub fn key(&self) -> Option<&str> { self.key.as_deref() }
    pub fn api(&self) -> Option<i32> { self.api }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FriendQueryParams {
    #[serde(default)]
    pub org: Option<i32>,
    #[serde(default)]
    pub from: Option<i32>,
    #[serde(default)]
    pub uid: Option<i32>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub api: Option<i32>,
}

impl PromoteQueryParams {
    pub fn uid(&self) -> Option<i32> { self.uid }
    pub fn key(&self) -> Option<&str> { self.key.as_deref() }
    pub fn api(&self) -> Option<i32> { self.api }
}

impl FriendQueryParams {
    pub fn uid(&self) -> Option<i32> { self.uid }
    pub fn key(&self) -> Option<&str> { self.key.as_deref() }
    pub fn api(&self) -> Option<i32> { self.api }
}

impl ChartsQueryParams {
    pub fn uid(&self) -> Option<i32> { self.uid }
    pub fn key(&self) -> Option<&str> { self.key.as_deref() }
    pub fn api(&self) -> Option<i32> { self.api }
}

impl DownloadQueryParams {
    pub fn uid(&self) -> Option<i32> { self.uid }
    pub fn key(&self) -> Option<&str> { self.key.as_deref() }
    pub fn api(&self) -> Option<i32> { self.api }
}

impl ResourceQueryParams {
    pub fn uid(&self) -> Option<i32> { self.uid }
    pub fn key(&self) -> Option<&str> { self.key.as_deref() }
    pub fn api(&self) -> Option<i32> { self.api }
}

impl SongQueryParams {
    pub fn uid(&self) -> Option<i32> { self.uid }
    pub fn key(&self) -> Option<&str> { self.key.as_deref() }
    pub fn api(&self) -> Option<i32> { self.api }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malody_mode_from_i32_all_values() {
        assert_eq!(MalodyMode::from_i32(-1), Some(MalodyMode::Any));
        assert_eq!(MalodyMode::from_i32(0), Some(MalodyMode::Key));
        assert_eq!(MalodyMode::from_i32(3), Some(MalodyMode::Catch));
        assert_eq!(MalodyMode::from_i32(4), Some(MalodyMode::Pad));
        assert_eq!(MalodyMode::from_i32(5), Some(MalodyMode::Taiko));
        assert_eq!(MalodyMode::from_i32(6), Some(MalodyMode::Ring));
        assert_eq!(MalodyMode::from_i32(7), Some(MalodyMode::Slide));
        assert_eq!(MalodyMode::from_i32(8), Some(MalodyMode::Live));
        assert_eq!(MalodyMode::from_i32(9), Some(MalodyMode::Cube));
    }

    #[test]
    fn malody_mode_from_i32_invalid() {
        assert_eq!(MalodyMode::from_i32(1), None);
        assert_eq!(MalodyMode::from_i32(2), None);
        assert_eq!(MalodyMode::from_i32(100), None);
        assert_eq!(MalodyMode::from_i32(-2), None);
    }

    #[test]
    fn malody_mode_is_osu_searchable() {
        assert!(MalodyMode::Any.is_osu_searchable());
        assert!(MalodyMode::Key.is_osu_searchable());
        assert!(MalodyMode::Catch.is_osu_searchable());
        assert!(MalodyMode::Taiko.is_osu_searchable());
        assert!(!MalodyMode::Pad.is_osu_searchable());
        assert!(!MalodyMode::Ring.is_osu_searchable());
        assert!(!MalodyMode::Slide.is_osu_searchable());
        assert!(!MalodyMode::Live.is_osu_searchable());
        assert!(!MalodyMode::Cube.is_osu_searchable());
    }

    #[test]
    fn malody_mode_to_osu_game_mode() {
        use rosu_v2::prelude::GameMode;
        assert_eq!(MalodyMode::Any.to_osu_game_mode(), GameMode::Mania);
        assert_eq!(MalodyMode::Key.to_osu_game_mode(), GameMode::Mania);
        assert_eq!(MalodyMode::Catch.to_osu_game_mode(), GameMode::Catch);
        assert_eq!(MalodyMode::Taiko.to_osu_game_mode(), GameMode::Taiko);
        assert_eq!(MalodyMode::Pad.to_osu_game_mode(), GameMode::Mania);
        assert_eq!(MalodyMode::Cube.to_osu_game_mode(), GameMode::Mania);
    }

    #[test]
    fn paged_response_serialization() {
        #[derive(Debug, Serialize, PartialEq)]
        struct TestItem { name: String }
        let resp = PagedResponse {
            code: 0,
            has_more: true,
            next: 50,
            data: vec![TestItem { name: "a".into() }],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"hasMore\":true"));
        assert!(json.contains("\"next\":50"));
        assert!(json.contains("\"code\":0"));
    }

    #[test]
    fn download_response_serialization() {
        let resp = DownloadResponse {
            code: 0,
            items: vec![DownloadItem {
                name: "test.osu".into(),
                hash: "abc123".into(),
                file: "http://localhost/api/store/1?type=chart".into(),
            }],
            sid: 100,
            cid: 1,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"sid\":100"));
        assert!(json.contains("\"cid\":1"));
        assert!(json.contains("\"hash\":\"abc123\""));
        assert!(json.contains("\"file\""));
    }

    #[test]
    fn list_query_params_defaults() {
        let params: ListQueryParams = serde_urlencoded::from_str("").unwrap();
        assert_eq!(params.word, None);
        assert_eq!(params.mode, MalodyMode::Any);
        assert_eq!(params.from, None);
    }

    #[test]
    fn list_query_params_with_values() {
        let params: ListQueryParams =
            serde_urlencoded::from_str("word=test&mode=0&from=50&uid=123&api=202310&key=abc")
                .unwrap();
        assert_eq!(params.word, Some("test".to_string()));
        assert_eq!(params.mode, MalodyMode::Key);
        assert_eq!(params.from, Some(50));
        assert_eq!(params.uid(), Some(123));
        assert_eq!(params.api(), Some(202310));
        assert_eq!(params.key(), Some("abc"));
    }
}
