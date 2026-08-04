use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::models::*;
use crate::services::{self, AppState};

pub async fn server_info(State(state): State<Arc<AppState>>) -> Json<ServerInfoResponse> {
    Json(services::server_info(&state))
}

pub async fn song_list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListQueryParams>,
) -> Json<PagedResponse<Song>> {
    if let Err(code) = check_list_auth(&state, &params) {
        return Json(PagedResponse {
            code,
            has_more: false,
            next: 0,
            data: vec![],
        });
    }
    Json(services::song_list(&state, &params).await)
}

pub async fn song_promote(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PromoteQueryParams>,
) -> Json<PagedResponse<Song>> {
    if let Err(code) = check_auth(&state, params.uid(), params.key(), params.api()) {
        return Json(PagedResponse {
            code,
            has_more: false,
            next: 0,
            data: vec![],
        });
    }
    Json(services::song_promote(&state, &params).await)
}

pub async fn song_friend(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FriendQueryParams>,
) -> Json<PagedResponse<Song>> {
    if let Err(code) = check_auth(&state, params.uid(), params.key(), params.api()) {
        return Json(PagedResponse {
            code,
            has_more: false,
            next: 0,
            data: vec![],
        });
    }
    Json(services::song_friend(&state, &params).await)
}

pub async fn charts_list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ChartsQueryParams>,
) -> Json<PagedResponse<Chart>> {
    if let Err(code) = check_auth(&state, params.uid(), params.key(), params.api()) {
        return Json(PagedResponse {
            code,
            has_more: false,
            next: 0,
            data: vec![],
        });
    }
    Json(services::charts_list(&state, params.sid).await)
}

pub async fn download_chart(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DownloadQueryParams>,
) -> Json<DownloadResponse> {
    if let Err(code) = check_auth(&state, params.uid(), params.key(), params.api()) {
        return Json(DownloadResponse {
            code,
            items: vec![],
            sid: 0,
            cid: params.cid,
        });
    }
    Json(
        services::download_chart(
            &state,
            params.cid,
            params.uid(),
            params.key(),
            params.api(),
        )
        .await,
    )
}

pub async fn song_query(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SongQueryParams>,
) -> Json<PagedResponse<Song>> {
    if let Err(code) = check_auth(&state, params.uid(), params.key(), params.api()) {
        return Json(PagedResponse {
            code,
            has_more: false,
            next: 0,
            data: vec![],
        });
    }
    Json(services::song_query(&state, &params).await)
}

pub async fn send_chart_resource(
    State(state): State<Arc<AppState>>,
    Path(cid): Path<i32>,
    Query(params): Query<ResourceQueryParams>,
) -> Response {
    // When `verify_client_auth` is enabled, enforce the same uid/key/api
    // signature on file fetches. The download endpoint forwards its auth
    // params into the returned file URLs so the client can replay them.
    if let Err(code) = check_auth(&state, params.uid(), params.key(), params.api()) {
        return (
            StatusCode::FORBIDDEN,
            format!("auth failed: code {code}"),
        )
            .into_response();
    }

    match services::send_resource(&state, cid, &params.resource_type).await {
        Ok((data, filename)) => {
            let content_type = match params.resource_type.as_str() {
                "chart" => "text/plain",
                "audio" => "audio/mpeg",
                "bg" => "image/jpeg",
                _ => "application/octet-stream",
            };
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, content_type),
                    (
                        header::CONTENT_DISPOSITION,
                        &*format!("attachment; filename=\"{}\"", filename),
                    ),
                ],
                data,
            )
                .into_response()
        }
        Err((status_code, message)) => {
            (StatusCode::from_u16(status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), message).into_response()
        }
    }
}

fn check_auth(state: &AppState, uid: Option<i32>, key: Option<&str>, api: Option<i32>) -> Result<(), i32> {
    let code = state.verify_client_auth(uid, key, api);
    if code == 0 { Ok(()) } else { Err(code) }
}

fn check_list_auth(state: &AppState, params: &ListQueryParams) -> Result<(), i32> {
    check_auth(state, params.uid(), params.key(), params.api())
}
