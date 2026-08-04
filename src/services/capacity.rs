use std::fs;
use std::path::Path;
use std::time::SystemTime;

use tracing::{info, warn};

use super::AppState;

/// Delete oldest cached .osz files until the total size of .osz files in the
/// tmp dir is within `malody.server.max_osz_size` (bytes). No-op when the
/// limit is 0 (unlimited). Best-effort: errors are logged, never propagated.
pub(super) fn enforce_osz_capacity(state: &AppState) {
    let limit = state
        .config
        .malody
        .server
        .max_osz_size
        .to_bytes()
        .unwrap_or(0);
    if limit == 0 {
        return;
    }

    let tmp_dir = &state.config.malody.server.tmp;
    let mut files = match scan_osz_files(tmp_dir) {
        Ok(files) => files,
        Err(e) => {
            warn!("Capacity check failed for tmp dir '{}': {:?}", tmp_dir, e);
            return;
        }
    };

    let total: u64 = files.iter().map(|f| f.size).sum();
    if total <= limit {
        return;
    }

    // Oldest (least recently accessed) first.
    files.sort_by_key(|f| f.modified);

    let mut freed: u64 = 0;
    for f in files {
        if total.saturating_sub(freed) <= limit {
            break;
        }
        match fs::remove_file(&f.path) {
            Ok(()) => {
                freed += f.size;
                info!(
                    "Capacity: deleted {} ({} bytes), total {} bytes over limit of {}",
                    f.path.display(),
                    f.size,
                    total,
                    limit
                );
            }
            Err(e) => warn!("Capacity: failed to delete {}: {:?}", f.path.display(), e),
        }
    }

    if freed > 0 {
        info!(
            "Capacity: freed {} bytes, {}/{} bytes of .osz cached",
            freed,
            total.saturating_sub(freed),
            limit
        );
    }
}

/// Update the mtime of a cached .osz file so the capacity manager keeps
/// recently served files (LRU). Errors are ignored.
pub(super) fn touch_osz(path: &Path) {
    let Ok(file) = fs::File::open(path) else { return };
    let _ = file.set_modified(SystemTime::now());
}

struct OszFile {
    path: std::path::PathBuf,
    size: u64,
    modified: SystemTime,
}

fn scan_osz_files(tmp_dir: &str) -> anyhow::Result<Vec<OszFile>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(tmp_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("osz"))
        {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if !meta.is_file() {
            continue;
        }
        files.push(OszFile {
            path,
            size: meta.len(),
            modified: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        });
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    use crate::config::Config;
    use crate::services::test_utils::dummy_state_with_config;

    use super::*;

    fn test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("malody-capacity-test-{name}"))
    }

    fn write_fake_osz(dir: &Path, name: &str, bytes: u64, age_secs: u64) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, vec![0u8; bytes as usize]).unwrap();
        let mtime = SystemTime::now() - Duration::from_secs(age_secs);
        let f = fs::File::open(&path).unwrap();
        f.set_modified(mtime).unwrap();
        path
    }

    fn state_with_limit(dir: &Path, limit: u64) -> AppState {
        let yaml = format!(
            r#"
server:
  port: 0
malody:
  server:
    api: 202310
    min: 202310
    welcome: "test server"
    tmp: "__TMPDIR__"
    max_osz_size: {limit}
  osu:
    clientID: 0
    clientSecret: ""
"#
        );
        let config: Config = serde_yaml::from_str(
            &yaml.replace("__TMPDIR__", &dir.display().to_string()),
        )
        .expect("test config parse");
        dummy_state_with_config(config)
    }

    fn clean_dir(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn deletes_oldest_until_under_limit() {
        let dir = test_dir("deletes_oldest");
        clean_dir(&dir);
        fs::create_dir_all(&dir).unwrap();

        let old = write_fake_osz(&dir, "1.osz", 100, 1000);
        let mid = write_fake_osz(&dir, "2.osz", 200, 500);
        let new = write_fake_osz(&dir, "3.osz", 300, 100);

        let state = state_with_limit(&dir, 350);
        enforce_osz_capacity(&state);

        assert!(!old.exists(), "oldest should be deleted");
        assert!(!mid.exists(), "second-oldest should be deleted");
        assert!(new.exists(), "newest should be kept");
        assert_eq!(fs::metadata(&new).unwrap().len(), 300);

        clean_dir(&dir);
    }

    #[test]
    fn limit_zero_keeps_everything() {
        let dir = test_dir("limit_zero");
        clean_dir(&dir);
        fs::create_dir_all(&dir).unwrap();

        let a = write_fake_osz(&dir, "1.osz", 100, 1000);
        let b = write_fake_osz(&dir, "2.osz", 200, 500);

        let state = state_with_limit(&dir, 0);
        enforce_osz_capacity(&state);

        assert!(a.exists());
        assert!(b.exists());

        clean_dir(&dir);
    }

    #[test]
    fn ignores_non_osz_files() {
        let dir = test_dir("ignores_non_osz");
        clean_dir(&dir);
        fs::create_dir_all(&dir).unwrap();

        write_fake_osz(&dir, "1.osz", 100, 1000);
        let keep = dir.join("notes.txt");
        fs::write(&keep, vec![0u8; 500]).unwrap();
        let part = dir.join("2.osz.part");
        fs::write(&part, vec![0u8; 500]).unwrap();

        let state = state_with_limit(&dir, 50);
        enforce_osz_capacity(&state);

        assert!(!dir.join("1.osz").exists());
        assert!(keep.exists(), "non-.osz files must not be deleted");
        assert!(part.exists(), ".osz.part files must not be deleted");

        clean_dir(&dir);
    }
}
