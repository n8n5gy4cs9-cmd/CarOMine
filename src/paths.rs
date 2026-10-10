//! Where saves and settings live: the OS per-user data dir (override with CAROMINE_DATA_DIR, e.g. `.` for portable/dev runs).
use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    let dir = if let Ok(d) = std::env::var("CAROMINE_DATA_DIR") { PathBuf::from(d) }
    else if cfg!(target_os = "macos") { std::env::var("HOME").map(|h| PathBuf::from(h).join("Library/Application Support/CarOMine")).unwrap_or_else(|_| PathBuf::from(".")) }
    else if cfg!(target_os = "windows") { std::env::var("APPDATA").map(|h| PathBuf::from(h).join("CarOMine")).unwrap_or_else(|_| PathBuf::from(".")) }
    else if let Ok(x) = std::env::var("XDG_DATA_HOME") { PathBuf::from(x).join("caromine") }
    else { std::env::var("HOME").map(|h| PathBuf::from(h).join(".local/share/caromine")).unwrap_or_else(|_| PathBuf::from(".")) };
    let _ = std::fs::create_dir_all(&dir);
    dir
}
