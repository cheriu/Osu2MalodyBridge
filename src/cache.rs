use moka::sync::Cache;
use rosu_v2::model::beatmap::BeatmapsetSearchResult;
use std::time::Duration;

use crate::models::{ListQueryParams, PromoteQueryParams};

/// Caches beatmap metadata by beatmap ID to avoid repeated osu!api calls.
pub struct BeatmapCache {
    cache: Cache<u32, BeatmapCacheEntry>,
}

#[derive(Clone)]
pub struct BeatmapCacheEntry {
    pub map_id: u32,
    pub mapset_id: u32,
    pub checksum: String,
    pub version: String,
    pub stars: f32,
    pub seconds_total: u32,
    pub creator_id: u32,
}

impl BeatmapCache {
    pub fn new() -> Self {
        let cache = Cache::builder()
            .max_capacity(2000)
            .time_to_live(Duration::from_secs(60 * 60))
            .build();
        Self { cache }
    }

    pub fn get(&self, beatmap_id: u32) -> Option<BeatmapCacheEntry> {
        self.cache.get(&beatmap_id)
    }

    pub fn put(&self, beatmap_id: u32, entry: BeatmapCacheEntry) {
        self.cache.insert(beatmap_id, entry);
    }
}

/// Caches search results for pagination. Keyed by the `from` offset the result
/// corresponds to, so sequential page requests hit the cache.
pub struct SearchCache {
    cache: Cache<u64, BeatmapsetSearchResult>,
}

impl SearchCache {
    pub fn new(max_capacity: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(60 * 60))
            .build();
        Self { cache }
    }

    pub fn get(&self, key: u64) -> Option<BeatmapsetSearchResult> {
        self.cache.get(&key)
    }

    pub fn put(&self, key: u64, result: BeatmapsetSearchResult) {
        self.cache.insert(key, result);
    }
}

/// Build a composite cache key from all search parameters.
fn list_cache_key(params: &ListQueryParams) -> u64 {
    let mut h = 0u64;
    for b in params.word.as_deref().unwrap_or("").bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as u64);
    }
    h = h.wrapping_mul(31).wrapping_add(params.org.unwrap_or(0) as u64);
    h = h.wrapping_mul(31).wrapping_add(params.mode.unwrap_or(-1) as u64);
    h = h.wrapping_mul(31).wrapping_add(params.lvge.unwrap_or(0) as u64);
    h = h.wrapping_mul(31).wrapping_add(params.lvle.unwrap_or(0) as u64);
    h = h.wrapping_mul(31).wrapping_add(params.beta.unwrap_or(0) as u64);
    h = h.wrapping_mul(31).wrapping_add(params.from.unwrap_or(0) as u64);
    h
}

fn promote_cache_key(params: &PromoteQueryParams) -> u64 {
    let org = params.org.unwrap_or(0) as u64;
    let mode = params.mode.unwrap_or(-1) as u64;
    let from = params.from.unwrap_or(0) as u64;
    org.wrapping_mul(31).wrapping_add(mode).wrapping_mul(31).wrapping_add(from)
}

/// List search result cache (for /api/store/list pagination)
pub struct ListSearchCache {
    cache: SearchCache,
}

impl ListSearchCache {
    pub fn new() -> Self {
        Self { cache: SearchCache::new(2000) }
    }

    pub fn get(&self, params: &ListQueryParams) -> Option<BeatmapsetSearchResult> {
        self.cache.get(list_cache_key(params))
    }

    pub fn put(&self, params: &ListQueryParams, result: BeatmapsetSearchResult) {
        self.cache.put(list_cache_key(params), result);
    }
}

/// Promote search result cache (for /api/store/promote pagination)
pub struct PromoteSearchCache {
    cache: SearchCache,
}

impl PromoteSearchCache {
    pub fn new() -> Self {
        Self { cache: SearchCache::new(2000) }
    }

    pub fn get(&self, params: &PromoteQueryParams) -> Option<BeatmapsetSearchResult> {
        self.cache.get(promote_cache_key(params))
    }

    pub fn put(&self, params: &PromoteQueryParams, result: BeatmapsetSearchResult) {
        self.cache.put(promote_cache_key(params), result);
    }
}
