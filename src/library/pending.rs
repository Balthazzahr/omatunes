use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static PENDING: OnceLock<Mutex<HashMap<PathBuf, bool>>> = OnceLock::new();

fn map() -> &'static Mutex<HashMap<PathBuf, bool>> {
    PENDING.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Queue a liked write for `path`. Overwrites any existing pending value (last toggle wins).
pub fn queue(path: PathBuf, liked: bool) {
    let mut m = map().lock().unwrap();
    m.insert(path, liked);
}

/// Take a single pending entry for `path` if present (removes it).
pub fn take(path: &PathBuf) -> Option<bool> {
    let mut m = map().lock().unwrap();
    m.remove(path)
}

/// Drain all pending entries (for exit).
pub fn drain_all() -> Vec<(PathBuf, bool)> {
    let mut m = map().lock().unwrap();
    m.drain().collect()
}

/// Peek without removing (for reconciliation).
pub fn snapshot() -> Vec<(PathBuf, bool)> {
    let m = map().lock().unwrap();
    m.iter().map(|(k, v)| (k.clone(), *v)).collect()
}

pub fn is_empty() -> bool {
    map().lock().unwrap().is_empty()
}

pub fn len() -> usize {
    map().lock().unwrap().len()
}

/// Flush a single pending entry via atomic copy→save→rename.
/// Returns true if flushed, false if nothing pending or error.
pub fn flush_one(path: &PathBuf) -> bool {
    let liked = match take(path) {
        Some(v) => v,
        None => return false,
    };
    if let Err(e) = crate::library::scanner::write_like_status_atomic(path, liked) {
        eprintln!("pending flush failed for {}: {}", path.display(), e);
        // Re-queue on failure so it can be retried next scan/exit
        queue(path.clone(), liked);
        return false;
    }
    true
}

/// Flush all pending entries (for non-playing bulk or exit).
pub fn flush_all() -> usize {
    let entries = drain_all();
    let mut n = 0;
    for (path, liked) in entries {
        if crate::library::scanner::write_like_status_atomic(&path, liked).is_ok() {
            n += 1;
        } else {
            // Re-queue failed
            queue(path, liked);
        }
    }
    n
}

/// Synchronous flush for exit (main.rs).
pub fn flush_sync() {
    // Drain and write synchronously (no spawn) so exit waits.
    let entries = drain_all();
    for (path, liked) in entries {
        if let Err(e) = crate::library::scanner::write_like_status_atomic(&path, liked) {
            eprintln!("pending flush_sync failed for {}: {}", path.display(), e);
        }
    }
}
