use rosu_v2::prelude::*;
use tracing::info;

use super::AppState;
use crate::cache::{list_search_key, promote_search_key, SearchChain, SearchChainCache};
use crate::models::*;

/// Fetch the page containing `from` from the search chain cache,
/// or fetch it from the osu! API and cache it.
async fn get_or_fetch_page(
    state: &AppState,
    chain_cache: &SearchChainCache,
    search_key: u64,
    from: i32,
    word: &str,
    is_promote: bool,
    mode: MalodyMode,
) -> anyhow::Result<BeatmapsetSearchResult> {
    let game_mode = mode.to_osu_game_mode();

    // Try to find the page in the existing chain
    if let Some(chain_last) = chain_cache.get_chain_last(search_key) {
        let chain = chain_cache.get_full_chain(search_key);

        let mut offset = 0i32;
        for (idx, page) in chain.iter().enumerate() {
            let page_size = page.mapsets.len() as i32;
            if from >= offset && from < offset + page_size {
                info!("Cache HIT: page {} for from={} (range {}..{})", idx, from, offset, offset + page_size);
                return Ok(page.clone());
            }
            offset += page_size;
        }

        let mut cursor = chain_last;
        while offset <= from {
            if !cursor.has_more() {
                return Ok(cursor);
            }
            info!("Walking forward with get_next() (offset={})", offset);
            match cursor.get_next(state.osu_client()).await {
                Some(Ok(next)) => {
                    cursor = next;
                    chain_cache.push_page(search_key, cursor.clone());
                }
                Some(Err(e)) => return Err(e.into()),
                None => return Ok(cursor),
            }
            offset += cursor.mapsets.len() as i32;
        }
        return Ok(cursor);
    }

    // No chain — fresh search
    info!("Fresh search for target from={} (mode={:?})", from, game_mode);
    let mut result = if is_promote {
        state.osu_client()
            .beatmapset_search()
            .mode(game_mode)
            .nsfw(false)
            .spotlights(true)
            .await?
    } else {
        let mut search = state.osu_client()
            .beatmapset_search()
            .mode(game_mode)
            .nsfw(false);
        if !word.is_empty() {
            search = search.query(word);
        }
        search.await?
    };

    let chain = SearchChain::new(result.clone());
    chain_cache.put_chain(search_key, chain);

    let mut offset = result.mapsets.len() as i32;
    while offset <= from && result.has_more() {
        info!("Walking to from={}, current offset={}", from, offset);
        match result.get_next(state.osu_client()).await {
            Some(Ok(next)) => {
                result = next;
                chain_cache.push_page(search_key, result.clone());
            }
            Some(Err(e)) => return Err(e.into()),
            None => break,
        }
        offset += result.mapsets.len() as i32;
    }

    Ok(result)
}

// ---------------------------------------------------------------------------
// Internal: song list
// ---------------------------------------------------------------------------

pub(super) async fn do_song_list(
    state: &AppState,
    params: &ListQueryParams,
    mode: MalodyMode,
) -> anyhow::Result<PagedResponse<Song>> {
    let from = params.from.unwrap_or(0);
    let org = params.org.unwrap_or(0);
    let word = params.word.as_deref().unwrap_or("");
    let search_key = list_search_key(params);

    // Look up the chain for this search. The chain stores pages indexed
    // sequentially (0, 1, 2, ...). We need to find which page contains `from`.
    // Strategy: walk the chain from page 0, tracking cumulative offsets.
    let result = get_or_fetch_page(state, &state.list_chain, search_key, from, word, false, mode).await?;

    let response = search_result_to_paged_songs(&result, from, org);
    Ok(response)
}

// ---------------------------------------------------------------------------
// Internal: promote
// ---------------------------------------------------------------------------

pub(super) async fn do_song_promote(
    state: &AppState,
    params: &PromoteQueryParams,
    mode: MalodyMode,
) -> anyhow::Result<PagedResponse<Song>> {
    let from = params.from.unwrap_or(0);
    let org = params.org.unwrap_or(0);
    let search_key = promote_search_key(params);

    let result = get_or_fetch_page(state, &state.promote_chain, search_key, from, "", true, mode).await?;

    let response = search_result_to_paged_songs(&result, from, org);
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

    let mode_bitmask = mapset
        .maps
        .as_ref()
        .map(|maps| {
            maps.iter()
                .map(|b| MalodyMode::from_osu_game_mode(b.mode))
                .fold(0i32, |acc, m| acc | (1i32 << (m as i32)))
        })
        .unwrap_or(0);

    let cover = &mapset.covers.list_2x;

    Song {
        sid: mapset.mapset_id as i32,
        cover: cover.clone(),
        length,
        bpm: mapset.bpm,
        title,
        artist: mapset.artist.clone(),
        mode: mode_bitmask,
        time: mapset.last_updated.unix_timestamp(),
    }
}

// ---------------------------------------------------------------------------
// Internal: query (lookup by sid or cid)
// ---------------------------------------------------------------------------

pub(super) async fn do_song_query(
    state: &AppState,
    sid: Option<i32>,
    cid: Option<i32>,
    org: i32,
) -> anyhow::Result<PagedResponse<Song>> {
    let mapset_id: u32 = if let Some(c) = cid {
        let beatmap: BeatmapExtended = state.osu_client().beatmap().map_id(c as u32).await?;
        beatmap.mapset_id
    } else if let Some(s) = sid {
        s as u32
    } else {
        anyhow::bail!("sid or cid must be provided")
    };

    let mapset: BeatmapsetExtended = if let Some(cached) = state.beatmapset_cache.get(mapset_id) {
        cached
    } else {
        let fetched: BeatmapsetExtended = state.osu_client().beatmapset(mapset_id).await?;
        state.beatmapset_cache.put(mapset_id, fetched.clone());
        fetched
    };
    let song = mapset_to_song(&mapset, org);

    Ok(PagedResponse {
        code: 0,
        has_more: false,
        next: 0,
        data: vec![song],
    })
}
