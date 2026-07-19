use serde::{Deserialize, Serialize};

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
            _ => None,
        }
    }
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
    #[serde(default)]
    pub mode: Option<i32>,
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
    #[serde(default)]
    pub mode: Option<i32>,
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
    #[serde(default)]
    pub mode: Option<i32>,
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

impl ListQueryParams {
    pub fn uid(&self) -> Option<i32> { self.uid }
    pub fn key(&self) -> Option<&str> { self.key.as_deref() }
    pub fn api(&self) -> Option<i32> { self.api }
}

impl PromoteQueryParams {
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
    }

    #[test]
    fn malody_mode_from_i32_invalid() {
        assert_eq!(MalodyMode::from_i32(1), None);
        assert_eq!(MalodyMode::from_i32(2), None);
        assert_eq!(MalodyMode::from_i32(9), None);
        assert_eq!(MalodyMode::from_i32(100), None);
        assert_eq!(MalodyMode::from_i32(-2), None);
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
        assert_eq!(params.mode, None);
        assert_eq!(params.from, None);
    }

    #[test]
    fn list_query_params_with_values() {
        let params: ListQueryParams =
            serde_urlencoded::from_str("word=test&mode=0&from=50&uid=123&api=202310&key=abc")
                .unwrap();
        assert_eq!(params.word, Some("test".to_string()));
        assert_eq!(params.mode, Some(0));
        assert_eq!(params.from, Some(50));
        assert_eq!(params.uid(), Some(123));
        assert_eq!(params.api(), Some(202310));
        assert_eq!(params.key(), Some("abc"));
    }
}
