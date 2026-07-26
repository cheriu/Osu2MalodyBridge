use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use osu2malody_store::config::Config;
use osu2malody_store::router::create_router;
use osu2malody_store::services::test_utils;

fn test_app(state: Arc<osu2malody_store::services::AppState>) -> axum::Router {
    create_router(state)
}

// ---------------------------------------------------------------------------
// Endpoints that don't need osu! API
// ---------------------------------------------------------------------------

#[tokio::test]
async fn server_info_returns_valid_json() {
    let state = Arc::new(test_utils::dummy_state());
    let app = test_app(state);

    let response = app
        .oneshot(Request::builder().uri("/api/store/info").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 0);
    assert_eq!(json["api"], 202310);
    assert_eq!(json["min"], 202310);
    assert_eq!(json["welcome"], "test server");
}

#[tokio::test]
async fn song_query_missing_params_returns_error() {
    let state = Arc::new(test_utils::dummy_state());
    let app = test_app(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/store/query")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], -1);
}

#[tokio::test]
async fn song_list_invalid_mode_returns_400() {
    let state = Arc::new(test_utils::dummy_state());
    let app = test_app(state);

    // mode=1 is not a valid MalodyMode — deserialization fails, axum returns 400.
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/store/list?mode=1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn song_list_malody_only_mode_returns_empty_success() {
    let state = Arc::new(test_utils::dummy_state());
    let app = test_app(state);

    // Pad (4) is a valid Malody mode with no osu! equivalent.
    // Should return empty success (code 0), not an error.
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/store/list?mode=4")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 0);
    assert_eq!(json["hasMore"], false);
    assert!(json["data"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn download_chart_unknown_cid_returns_code_minus_2() {
    let state = Arc::new(test_utils::dummy_state());
    let app = test_app(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/store/download?cid=999999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], -2);
    assert_eq!(json["cid"], 999999);
}

#[tokio::test]
async fn resource_unknown_cid_returns_error() {
    let state = Arc::new(test_utils::dummy_state());
    let app = test_app(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/store/999999?type=chart")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // osu! client unavailable → beatmap lookup fails → 500 or 404
    assert!(!response.status().is_success());
}

#[tokio::test]
async fn client_auth_rejected_when_verify_enabled_and_params_missing() {
    let config = {
        let yaml = r#"
server:
  port: 0
malody:
  server:
    api: 202310
    min: 202310
    welcome: ""
    tmp: "/tmp"
    verify_client_auth: true
  osu:
    clientID: 0
    clientSecret: ""
"#;
        serde_yaml::from_str::<Config>(yaml).unwrap()
    };

    let state = Arc::new(test_utils::dummy_state_with_config(config));
    let app = test_app(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/store/list?mode=5")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], -1);
}

#[tokio::test]
async fn client_auth_passes_and_mode_filter_applies() {
    let config = {
        let yaml = r#"
server:
  port: 0
malody:
  server:
    api: 202310
    min: 202310
    welcome: ""
    tmp: "/tmp"
    verify_client_auth: true
  osu:
    clientID: 0
    clientSecret: ""
"#;
        serde_yaml::from_str::<Config>(yaml).unwrap()
    };

    let state = Arc::new(test_utils::dummy_state_with_config(config));
    let app = test_app(state);

    // mode=4 (Pad) is a valid Malody mode with no osu! equivalent.
    // Auth should pass, then mode logic returns empty success.
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/store/list?mode=4&uid=1&key=any&api=202310")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 0); // auth passed, mode has no osu! results
}

// ---------------------------------------------------------------------------
// Live tests — require OSU_CLIENT_ID and OSU_CLIENT_SECRET env vars
// Same env vars as the server config (application.yml / Config::load).
// Run with: OSU_CLIENT_ID=xxx OSU_CLIENT_SECRET=xxx cargo test -- --ignored
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires OSU_CLIENT_ID and OSU_CLIENT_SECRET env vars"]
async fn live_search_beatmaps_returns_results() {
    let state = match test_utils::live_state().await {
        Some(s) => Arc::new(s),
        None => return, // skip if env vars not set
    };
    let app = test_app(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/store/list?mode=0&word=xi")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 10 * 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["code"], 0);
    assert!(!json["data"].as_array().unwrap().is_empty(), "search should return results");
    // Verify song structure
    let song = &json["data"][0];
    assert!(song["sid"].as_i64().is_some());
    assert!(song["title"].as_str().is_some());
    assert!(song["artist"].as_str().is_some());
}

#[tokio::test]
#[ignore = "requires OSU_CLIENT_ID and OSU_CLIENT_SECRET env vars"]
async fn live_promote_returns_spotlighted_maps() {
    let state = match test_utils::live_state().await {
        Some(s) => Arc::new(s),
        None => return,
    };
    let app = test_app(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/store/promote?org=0&mode=-1&from=0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 10 * 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["code"], 0);
    assert!(json["hasMore"].as_bool().is_some());
    assert!(json["next"].as_i64().is_some());
}

#[tokio::test]
#[ignore = "requires OSU_CLIENT_ID and OSU_CLIENT_SECRET env vars"]
async fn live_charts_returns_beatmap_difficulties() {
    let state = match test_utils::live_state().await {
        Some(s) => Arc::new(s),
        None => return,
    };
    let app = test_app(state);

    // Use a known beatmapset ID with mania maps (e.g., 1980361)
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/store/charts?sid=1980361")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 10 * 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["code"], 0);
    let charts = json["data"].as_array().unwrap();
    assert!(!charts.is_empty(), "beatmapset should have charts");
    let chart = &charts[0];
    assert!(chart["cid"].as_i64().is_some());
    assert!(chart["version"].as_str().is_some());
    assert!(chart["level"].as_i64().is_some());
}

#[tokio::test]
#[ignore = "requires OSU_CLIENT_ID and OSU_CLIENT_SECRET env vars"]
async fn live_download_returns_items_for_valid_beatmap() {
    let state = match test_utils::live_state().await {
        Some(s) => Arc::new(s),
        None => return,
    };
    let app = test_app(state);

    // Step 1: search to find a real beatmap
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/store/list?mode=0&word=xi")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), 10 * 1024).await.unwrap();
    let search: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(search["code"], 0, "search should succeed");

    let songs = search["data"].as_array().unwrap();
    assert!(!songs.is_empty(), "need at least one search result");
    let sid = songs[0]["sid"].as_i64().unwrap() as i32;

    // Step 2: get charts for that song
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/store/charts?sid={}", sid))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), 10 * 1024).await.unwrap();
    let charts: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(charts["code"], 0, "charts should succeed for sid={}", sid);

    let chart_list = charts["data"].as_array().unwrap();
    assert!(!chart_list.is_empty());
    let cid = chart_list[0]["cid"].as_i64().unwrap() as i32;

    // Step 3: download by cid
    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/store/download?cid={}", cid))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 50 * 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["code"], 0, "download should succeed for cid={}", cid);
    let items = json["items"].as_array().unwrap();
    assert_eq!(items.len(), 3, "should have chart, audio, bg items");
    assert!(items[0]["file"].as_str().unwrap().contains("type=chart"));
    assert!(items[1]["file"].as_str().unwrap().contains("type=audio"));
    assert!(items[2]["file"].as_str().unwrap().contains("type=bg"));
}

#[tokio::test]
#[ignore = "requires OSU_CLIENT_ID and OSU_CLIENT_SECRET env vars"]
async fn live_list_pagination_works() {
    let state = match test_utils::live_state().await {
        Some(s) => Arc::new(s),
        None => return,
    };
    let app = test_app(state);

    // Page 1
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/store/list?mode=0&word=xi&from=0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), 10 * 1024).await.unwrap();
    let page1: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(page1["code"], 0);

    if page1["hasMore"].as_bool() == Some(true) {
        let next_from = page1["next"].as_i64().unwrap();

        // Page 2
        let response = app
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/store/list?mode=0&word=xi&from={}", next_from))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), 10 * 1024).await.unwrap();
        let page2: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(page2["code"], 0);
        assert!(!page2["data"].as_array().unwrap().is_empty());
    }
}
