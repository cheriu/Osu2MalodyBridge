use rosu_v2::prelude::*;

use super::AppState;
use crate::models::*;

// ---------------------------------------------------------------------------
// Internal: charts
// ---------------------------------------------------------------------------

pub(super) async fn do_charts_list(state: &AppState, sid: i32) -> anyhow::Result<PagedResponse<Chart>> {
    let mapset_id = sid as u32;

    let mapset = if let Some(cached) = state.beatmapset_cache.get(mapset_id) {
        cached
    } else {
        let fetched: BeatmapsetExtended = state.osu_client().beatmapset(mapset_id).await?;
        state.beatmapset_cache.put(mapset_id, fetched.clone());
        fetched
    };

    let charts: Vec<Chart> = mapset
        .maps
        .as_ref()
        .map(|maps| {
            maps.iter()
                .filter(|b| b.mode != GameMode::Osu)
                .map(beatmap_ext_to_chart)
                .collect()
        })
        .unwrap_or_default();

    Ok(PagedResponse {
        code: 0,
        has_more: false,
        next: 0,
        data: charts,
    })
}

pub(super) fn beatmap_ext_to_chart(beatmap: &BeatmapExtended) -> Chart {
    let level = (beatmap.stars as f64 + 0.5).floor() as i32;
    let malody_mode = MalodyMode::from_osu_game_mode(beatmap.mode);

    Chart {
        cid: beatmap.map_id as i32,
        uid: beatmap.creator_id as i32,
        creator: beatmap.creator_id.to_string(),
        version: format!("{} R.{}", beatmap.version, beatmap.stars),
        level,
        length: beatmap.seconds_total as i32,
        chart_type: 1,
        size: 0,
        mode: malody_mode as i32,
    }
}
