use std::fs;
use std::io::Read;
use std::path::Path;

use md5::{Digest, Md5};
use rosu_v2::prelude::BeatmapExtended;
use tracing::info;
use zip::ZipArchive;

use super::AppState;
use super::API_BASE_PATH;
use crate::cache::BeatmapCacheEntry;
use crate::models::*;
use crate::osu_parser;
use crate::services::capacity;

// ---------------------------------------------------------------------------
// Internal: download
// ---------------------------------------------------------------------------

pub(super) async fn do_download_with_entry(
    state: &AppState,
    beatmap_entry: &BeatmapCacheEntry,
) -> anyhow::Result<Vec<DownloadItem>> {
    let mapset_id = beatmap_entry.mapset_id;

    let osz_filename = format!("{}.osz", mapset_id);
    let osz_path = Path::new(&state.config.malody.server.tmp).join(&osz_filename);

    if !osz_path.exists() {
        download_osz(state, mapset_id, &osz_path).await?;
        // Evict oldest .osz files once the cache exceeds the configured size.
        // The freshly written file has the newest mtime, so it is never evicted.
        capacity::enforce_osz_capacity(state);
    } else {
        capacity::touch_osz(&osz_path);
        // Validate existing .osz: stale JSON error responses from mirrors
        if let Ok(bytes) = fs::read(&osz_path) {
            if bytes.starts_with(b"{") {
                info!("Removing stale corrupted .osz for mapset {}", mapset_id);
                fs::remove_file(&osz_path)?;
                download_osz(state, mapset_id, &osz_path).await?;
                capacity::enforce_osz_capacity(state);
            }
        }
    }

    let scheme = if state.config.server.tls_cert.is_some() { "https" } else { "http" };
    let port = state.config.server.port;

    let h = &state.config.server.host;
    let host_port = if h.parse::<std::net::Ipv6Addr>().is_ok() {
        format!("[{}]:{}", h, port)
    } else if scheme == "https" && port == 443 || scheme == "http" && port == 80 {
        h.to_string()
    } else {
        format!("{}:{}", h, port)
    };
    let base_url = format!("{}://{}/{}", scheme, host_port, API_BASE_PATH);

    build_download_items(&osz_path, beatmap_entry, &base_url)
}

pub(super) fn build_download_items(
    osz_path: &Path,
    beatmap: &BeatmapCacheEntry,
    base_url: &str,
) -> anyhow::Result<Vec<DownloadItem>> {
    let file = fs::File::open(osz_path)?;
    let mut archive = ZipArchive::new(file)?;

    let osu_content = find_osu_by_checksum(&mut archive, &beatmap.checksum)?;
    let osu_str = String::from_utf8_lossy(&osu_content);

    let (audio_name, bg_name) = osu_parser::parse_audio_and_background(&osu_str);
    let audio_name = audio_name.ok_or_else(|| anyhow::anyhow!("Audio filename not found"))?;
    let bg_name = bg_name.ok_or_else(|| anyhow::anyhow!("Background filename not found"))?;

    let audio_hash = md5_of_zip_entry(osz_path, &audio_name)?;
    let bg_hash = md5_of_zip_entry(osz_path, &bg_name)?;

    Ok(vec![
        DownloadItem {
            name: format!("{} R.{}.osu", beatmap.version, beatmap.stars),
            hash: beatmap.checksum.clone(),
            file: format!("{}/{}?type=chart", base_url, beatmap.map_id),
        },
        DownloadItem {
            name: audio_name,
            hash: audio_hash,
            file: format!("{}/{}?type=audio", base_url, beatmap.map_id),
        },
        DownloadItem {
            name: bg_name,
            hash: bg_hash,
            file: format!("{}/{}?type=bg", base_url, beatmap.map_id),
        },
    ])
}

pub(super) fn find_osu_by_checksum<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    target_checksum: &str,
) -> anyhow::Result<Vec<u8>> {
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_lowercase();
        if name.ends_with(".osu") {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            let hash = format!("{:x}", Md5::digest(&buf));
            if hash == target_checksum {
                return Ok(buf);
            }
        }
    }
    anyhow::bail!("No .osu file matching checksum {}", target_checksum)
}

pub(super) fn md5_of_zip_entry(zip_path: &Path, target_name: &str) -> anyhow::Result<String> {
    let file = fs::File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.name().eq_ignore_ascii_case(target_name) && !entry.is_dir() {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            return Ok(format!("{:x}", Md5::digest(&buf)));
        }
    }
    anyhow::bail!("Entry '{}' not found in zip", target_name)
}

// ---------------------------------------------------------------------------
// Internal: send resource
// ---------------------------------------------------------------------------

pub(super) async fn do_send_resource(
    state: &AppState,
    cid: i32,
    resource_type: &str,
) -> Result<(Vec<u8>, String), (u16, String)> {
    let beatmap_entry = get_beatmap_entry(state, cid as u32)
        .await
        .map_err(|e| (500, format!("Beatmap not found: {:?}", e)))?;

    let osz_path = Path::new(&state.config.malody.server.tmp)
        .join(format!("{}.osz", beatmap_entry.mapset_id));

    if !osz_path.exists() {
        return Err((404, "osz file not found".into()));
    }

    // Keep track of recent access so the capacity manager evicts LRU first.
    capacity::touch_osz(&osz_path);

    let file =
        fs::File::open(&osz_path).map_err(|e| (500, format!("Cannot open osz: {:?}", e)))?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| (500, format!("Cannot read zip: {:?}", e)))?;

    let osu_content = find_osu_by_checksum(&mut archive, &beatmap_entry.checksum)
        .map_err(|e| (404, format!("Chart file not found: {:?}", e)))?;

    match resource_type {
        "chart" => {
            let filename = format!("{} R.{}.osu", beatmap_entry.version, beatmap_entry.stars);
            Ok((osu_content, filename))
        }
        "audio" => {
            let osu_str = String::from_utf8_lossy(&osu_content);
            let audio_name = osu_parser::parse_audio_filename(&osu_str)
                .ok_or_else(|| (404, "Audio filename not found".into()))?;

            let file =
                fs::File::open(&osz_path).map_err(|e| (500, format!("{}", e)))?;
            let mut archive2 =
                ZipArchive::new(file).map_err(|e| (500, format!("{}", e)))?;

            // Collect all matching entries to detect duplicates
            let mut matches: Vec<Vec<u8>> = Vec::new();
            for i in 0..archive2.len() {
                let mut entry =
                    archive2.by_index(i).map_err(|e| (500, format!("{}", e)))?;
                if entry.name().eq_ignore_ascii_case(&audio_name) && !entry.is_dir() {
                    let mut buf = Vec::new();
                    entry
                        .read_to_end(&mut buf)
                        .map_err(|e| (500, format!("{}", e)))?;
                    matches.push(buf);
                }
            }

            match matches.len() {
                0 => Err((404, "audio file not found".into())),
                1 => Ok((matches.into_iter().next().unwrap(), audio_name)),
                _ => Err((409, "multiple audio file candidates".into())),
            }
        }
        "bg" => {
            let osu_str = String::from_utf8_lossy(&osu_content);
            let bg_name = osu_parser::parse_background_filename(&osu_str)
                .ok_or_else(|| (404, "Background filename not found".into()))?;

            let file =
                fs::File::open(&osz_path).map_err(|e| (500, format!("{}", e)))?;
            let mut archive2 =
                ZipArchive::new(file).map_err(|e| (500, format!("{}", e)))?;

            let mut matches: Vec<Vec<u8>> = Vec::new();
            for i in 0..archive2.len() {
                let mut entry =
                    archive2.by_index(i).map_err(|e| (500, format!("{}", e)))?;
                if entry.name().eq_ignore_ascii_case(&bg_name) && !entry.is_dir() {
                    let mut buf = Vec::new();
                    entry
                        .read_to_end(&mut buf)
                        .map_err(|e| (500, format!("{}", e)))?;
                    matches.push(buf);
                }
            }

            match matches.len() {
                0 => Err((404, "bg file not found".into())),
                1 => Ok((matches.into_iter().next().unwrap(), bg_name)),
                _ => Err((409, "multiple bg file candidates".into())),
            }
        }
        _ => Err((400, "Invalid type parameter".into())),
    }
}

// ---------------------------------------------------------------------------
// .osz download helper
// ---------------------------------------------------------------------------

pub(super) async fn download_osz(state: &AppState, mapset_id: u32, osz_path: &Path) -> anyhow::Result<()> {
    let url = state.config.malody.server.mirror.url_for(mapset_id);
    info!("Downloading .osz from {}", url);

    let response = state.http_client.get(&url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Download failed with status {} for mapset {}",
            response.status(),
            mapset_id
        );
    }

    let bytes = response.bytes().await?;

    // Detect JSON error responses from mirrors (returned instead of binary .osz)
    if bytes.starts_with(b"{") {
        let preview = String::from_utf8_lossy(&bytes[..bytes.len().min(200)]);
        anyhow::bail!("Mirror returned error for mapset {}: {}", mapset_id, preview);
    }

    // Verify the response looks like a zip file (starts with PK)
    if !bytes.starts_with(b"PK") {
        anyhow::bail!(
            "Downloaded file for mapset {} is not a valid .osz (starts with 0x{:02x}{:02x})",
            mapset_id,
            bytes.first().copied().unwrap_or(0),
            bytes.get(1).copied().unwrap_or(0),
        );
    }

    let parent = osz_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid tmp path"))?;
    fs::create_dir_all(parent)?;

    // Write to a temp file, then atomically rename to the final path.
    // Prevents concurrent requests from observing a partially-written file
    // and avoids corruption if the write is interrupted.
    let tmp_path = osz_path.with_extension("osz.part");
    // Clean up stale .part from a previous interrupted download
    let _ = fs::remove_file(&tmp_path);
    fs::write(&tmp_path, &bytes)?;
    fs::rename(&tmp_path, osz_path)?;

    info!("Downloaded .osz for mapset {} ({} bytes)", mapset_id, bytes.len());

    Ok(())
}

// ---------------------------------------------------------------------------
// Beatmap cache
// ---------------------------------------------------------------------------

pub(super) async fn get_beatmap_entry(
    state: &AppState,
    beatmap_id: u32,
) -> anyhow::Result<BeatmapCacheEntry> {
    if let Some(entry) = state.beatmap_cache.get(beatmap_id) {
        return Ok(entry);
    }

    let osu = state
        .osu_opt()
        .ok_or_else(|| anyhow::anyhow!("osu! API client not available"))?;
    let beatmap: BeatmapExtended = osu.beatmap().map_id(beatmap_id).await?;

    let checksum = beatmap.checksum.clone().unwrap_or_default();

    let entry = BeatmapCacheEntry {
        map_id: beatmap.map_id,
        mapset_id: beatmap.mapset_id,
        checksum,
        version: beatmap.version.clone(),
        stars: beatmap.stars,
        seconds_total: beatmap.seconds_total,
        creator_id: beatmap.creator_id,
    };

    state.beatmap_cache.put(beatmap_id, entry.clone());
    Ok(entry)
}
