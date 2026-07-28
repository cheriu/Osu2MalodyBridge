use moka::sync::Cache;
use rosu_v2::model::beatmap::{BeatmapsetExtended, BeatmapsetSearchResult};
use std::sync::{Arc, Mutex};
use std::time::Duration;

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

/// Caches full beatmapset metadata by mapset ID.
pub struct BeatmapsetCache {
    cache: Cache<u32, BeatmapsetExtended>,
}

impl BeatmapsetCache {
    pub fn new() -> Self {
        let cache = Cache::builder()
            .max_capacity(2000)
            .time_to_live(Duration::from_secs(60 * 60))
            .build();
        Self { cache }
    }

    pub fn get(&self, mapset_id: u32) -> Option<BeatmapsetExtended> {
        self.cache.get(&mapset_id)
    }

    pub fn put(&self, mapset_id: u32, mapset: BeatmapsetExtended) {
        self.cache.insert(mapset_id, mapset);
    }
}

// ---------------------------------------------------------------------------
// Search chain cache — maps search params to a sequence of paginated pages.
// Uses rosu-v2's internal cursor handling via get_next().
// ---------------------------------------------------------------------------

/// A chain of search result pages for a single query.
/// Pages[0] = page 1, Pages[1] = page 2, etc.
/// Each entry is the result for that page (which also contains the cursor
/// for the NEXT page).
#[derive(Clone)]
pub struct SearchChain {
    pages: Vec<BeatmapsetSearchResult>,
}

impl SearchChain {
    pub fn new(first_page: BeatmapsetSearchResult) -> Self {
        Self { pages: vec![first_page] }
    }

    pub fn last(&self) -> &BeatmapsetSearchResult {
        self.pages.last().unwrap()
    }

    pub fn push(&mut self, page: BeatmapsetSearchResult) {
        self.pages.push(page);
    }

    pub fn get(&self, index: usize) -> Option<&BeatmapsetSearchResult> {
        self.pages.get(index)
    }

    pub fn has_more(&self) -> bool {
        self.last().has_more()
    }
}

/// Thread-safe search chain cache.
pub struct SearchChainCache {
    cache: Cache<u64, Arc<Mutex<SearchChain>>>,
}

impl SearchChainCache {
    pub fn new() -> Self {
        let cache = Cache::builder()
            .max_capacity(2000)
            .time_to_live(Duration::from_secs(60 * 60))
            .build();
        Self { cache }
    }

    /// Get a cached page by search key and page index.
    pub fn get_page(&self, key: u64, page: usize) -> Option<BeatmapsetSearchResult> {
        self.cache.get(&key)?.lock().ok()?.get(page).cloned()
    }

    /// Put a new chain (first page) into the cache.
    pub fn put_chain(&self, key: u64, chain: SearchChain) {
        self.cache.insert(key, Arc::new(Mutex::new(chain)));
    }

    /// Append a page to an existing chain. Returns the chain's last page cursor
    /// so the caller can call get_next().
    pub fn get_chain_last(&self, key: u64) -> Option<BeatmapsetSearchResult> {
        let chain_arc = self.cache.get(&key)?;
        let guard = chain_arc.lock().ok()?;
        Some(guard.last().clone())
    }

    /// Get the full chain (all pages) for a search key.
    pub fn get_full_chain(&self, key: u64) -> Vec<BeatmapsetSearchResult> {
        let Some(chain_arc) = self.cache.get(&key) else { return vec![] };
        let Ok(guard) = chain_arc.lock() else { return vec![] };
        guard.pages.clone()
    }

    /// Add a page to the chain.
    pub fn push_page(&self, key: u64, page: BeatmapsetSearchResult) {
        if let Some(chain) = self.cache.get(&key) {
            if let Ok(mut chain) = chain.lock() {
                chain.push(page);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Cache key builders — hash search params (excluding `from`)
// ---------------------------------------------------------------------------

/// Build a cache key from search parameters (without `from` offset).
pub fn list_search_key(params: &crate::models::ListQueryParams) -> u64 {
    let mut h = 0u64;
    for b in params.word.as_deref().unwrap_or("").bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as u64);
    }
    h = h.wrapping_mul(31).wrapping_add(params.org.unwrap_or(0) as u64);
    h = h.wrapping_mul(31).wrapping_add(params.mode as i32 as u64);
    h = h.wrapping_mul(31).wrapping_add(params.lvge.unwrap_or(0) as u64);
    h = h.wrapping_mul(31).wrapping_add(params.lvle.unwrap_or(0) as u64);
    h = h.wrapping_mul(31).wrapping_add(params.beta.unwrap_or(0) as u64);
    h
}

pub fn promote_search_key(params: &crate::models::PromoteQueryParams) -> u64 {
    let org = params.org.unwrap_or(0) as u64;
    let mode = params.mode as i32 as u64;
    org.wrapping_mul(31).wrapping_add(mode)
}

pub fn friend_search_key(params: &crate::models::FriendQueryParams) -> u64 {
    params.org.unwrap_or(0) as u64
}
