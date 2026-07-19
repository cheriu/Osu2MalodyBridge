use rosu_v2::prelude::*;
use tracing::{error, info};

use super::AppState;
use crate::models::*;

// ---------------------------------------------------------------------------
// Internal: song list
// ---------------------------------------------------------------------------

pub(super) async fn do_song_list(
    state: &AppState,
    params: &ListQueryParams,
) -> anyhow::Result<PagedResponse<Song>> {
    let from = params.from.unwrap_or(0);
    let org = params.org.unwrap_or(0);
    let word = params.word.as_deref().unwrap_or("");

    // Try to get the page starting at `from`. If we have a result cached
    // for this `from` value, it was pre-cached from the previous page
    // request. Call get_next() to advance to the actual page at this offset.
    let result = if let Some(cached) = state.list_cache.get(params) {
        info!("List: cache hit for from={}, calling get_next()", from);
        match cached.get_next(state.osu_client()).await {
            Some(Ok(next)) => {
                info!("List: get_next() returned {} mapsets", next.mapsets.len());
                next
            }
            other => {
                error!("List: get_next() failed for from={}: {:?}", from, other.as_ref().map(|r| r.as_ref().map(|_| ())));
                return Err(anyhow::anyhow!("get_next failed"));
            }
        }
    } else {
        let mut search = state.osu_client().beatmapset_search();
        search = search.mode(GameMode::Mania).nsfw(false);
        if !word.is_empty() {
            search = search.query(word);
        }

        let r = search.await?;
        info!(
            "List: fresh search for '{}', got {} mapsets (total={})",
            word,
            r.mapsets.len(),
            r.total
        );
        r
    };

    let response = search_result_to_paged_songs(&result, from, org);

    // Pre-cache this result for the NEXT page's `from` value,
    // mirroring the Kotlin cursor_string caching pattern.
    if result.has_more() {
        let next_from = from + result.mapsets.len() as i32;
        let mut next_params = params.clone();
        next_params.from = Some(next_from);
        state.list_cache.put(&next_params, result);
    }

    Ok(response)
}

// ---------------------------------------------------------------------------
// Internal: promote
// ---------------------------------------------------------------------------

pub(super) async fn do_song_promote(
    state: &AppState,
    params: &PromoteQueryParams,
) -> anyhow::Result<PagedResponse<Song>> {
    let from = params.from.unwrap_or(0);
    let org = params.org.unwrap_or(0);

    let result = if let Some(cached) = state.promote_cache.get(params) {
        info!("Promote: cache hit for from={}, calling get_next()", from);
        match cached.get_next(state.osu_client()).await {
            Some(Ok(next)) => {
                info!("Promote: get_next() returned {} mapsets", next.mapsets.len());
                next
            }
            other => {
                error!("Promote: get_next() failed for from={}: {:?}", from, other.as_ref().map(|r| r.as_ref().map(|_| ())));
                return Err(anyhow::anyhow!("get_next failed"));
            }
        }
    } else {
        let r = state
            .osu_client()
            .beatmapset_search()
            .mode(GameMode::Mania)
            .nsfw(false)
            .spotlights(true)
            .await?;
        info!(
            "Promote: fresh search, got {} mapsets (total={})",
            r.mapsets.len(),
            r.total
        );
        r
    };

    let response = search_result_to_paged_songs(&result, from, org);

    if result.has_more() {
        let next_from = from + result.mapsets.len() as i32;
        let mut next_params = params.clone();
        next_params.from = Some(next_from);
        state.promote_cache.put(&next_params, result);
    }

    Ok(response)
}

// ---------------------------------------------------------------------------
// Mapping: search result -> PagedResponse<Song>
// ---------------------------------------------------------------------------

pub(super) fn search_result_to_paged_songs(
    result: &BeatmapsetSearchResult,
    from: i32,
    org: i32,
) -> PagedResponse<Song> {
    if result.total == 0 {
        return PagedResponse {
            code: 1,
            has_more: false,
            next: 0,
            data: vec![],
        };
    }

    let songs: Vec<Song> = result.mapsets.iter().map(|m| mapset_to_song(m, org)).collect();
    let has_more = result.has_more();
    let next = from + songs.len() as i32;

    PagedResponse {
        code: 0,
        has_more,
        next,
        data: songs,
    }
}

pub(super) fn mapset_to_song(mapset: &BeatmapsetExtended, org: i32) -> Song {
    let title = if org == 0 {
        mapset.title.clone()
    } else {
        mapset
            .title_unicode
            .clone()
            .unwrap_or_else(|| mapset.title.clone())
    };

    let length = mapset
        .maps
        .as_ref()
        .and_then(|maps| maps.iter().map(|b| b.seconds_total).min())
        .unwrap_or(0) as i32;

    let cover = &mapset.covers.list_2x;

    Song {
        sid: mapset.mapset_id as i32,
        cover: cover.clone(),
        length,
        bpm: mapset.bpm,
        title,
        artist: mapset.artist.clone(),
        mode: 0,
        time: mapset.last_updated.unix_timestamp(),
    }
}

