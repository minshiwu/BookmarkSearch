use std::{fs, path::{Path, PathBuf}, time::SystemTime};
use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::channel;
use wry::application::event_loop::EventLoopProxy;

use crate::AppEvent;

#[derive(Debug, Clone, Serialize)]
pub struct BookmarkItem {
    pub title: String,
    pub url: String,
    pub browser: String,
    pub profile: String,
}

#[derive(Debug, Deserialize)]
struct ChromeBookmarksRoot {
    roots: ChromeRoots,
}

#[derive(Debug, Deserialize)]
struct ChromeRoots {
    bookmark_bar: Option<ChromeNode>,
    other: Option<ChromeNode>,
    synced: Option<ChromeNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
enum ChromeNode {
    Folder { name: String, children: Option<Vec<ChromeNode>> },
    Url { name: String, url: String },
}

pub fn scan_all_browsers() -> Vec<BookmarkItem> {
    let mut items = Vec::new();
    // Chrome-based browsers
    scan_chrome_like("Chrome", chrome_paths(), &mut items);
    scan_chrome_like("Edge", edge_paths(), &mut items);
    scan_opera_like("Opera", opera_paths(), &mut items);
    scan_opera_like("OperaGX", opera_gx_paths(), &mut items);
    items
}

pub fn spawn_watchers(_proxy: EventLoopProxy<crate::AppEvent>) {
    #[cfg(windows)]
    {
        let paths = all_bookmark_files();
        std::thread::spawn(move || {
            let (tx, rx) = channel();
            let mut watcher = notify::recommended_watcher(tx).expect("watcher");
            for p in paths {
                let _ = watcher.watch(&p, RecursiveMode::NonRecursive);
            }
            while let Ok(event) = rx.recv() {
                if matches!(event.kind, EventKind::Modify(_)|EventKind::Create(_)) {
                    let _ = _proxy.send_event(crate::AppEvent::Search("".into()));
                }
            }
        });
    }
}

fn scan_chrome_like(browser: &str, base: Option<PathBuf>, out: &mut Vec<BookmarkItem>) {
    let Some(base) = base else { return; };
    // default profile
    let mut candidates = Vec::new();
    candidates.push(base.join("Default").join("Bookmarks"));
    // additional profiles
    if let Ok(entries) = fs::read_dir(&base) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("Profile ") {
                candidates.push(e.path().join("Bookmarks"));
            }
        }
    }
    for file in candidates {
        if file.exists() { parse_chrome_bookmarks(browser, &file, out); }
    }
}

fn scan_opera_like(browser: &str, base: Option<PathBuf>, out: &mut Vec<BookmarkItem>) {
    let Some(base) = base else { return; };
    let file = base.join("Bookmarks");
    if file.exists() { parse_chrome_bookmarks(browser, &file, out); }
}

fn parse_chrome_bookmarks(browser: &str, path: &Path, out: &mut Vec<BookmarkItem>) {
    if let Ok(text) = fs::read_to_string(path) {
        if let Ok(root) = serde_json::from_str::<ChromeBookmarksRoot>(&text) {
            if let Some(node) = root.roots.bookmark_bar { collect_node(browser, "Default", node, out); }
            if let Some(node) = root.roots.other { collect_node(browser, "Default", node, out); }
            if let Some(node) = root.roots.synced { collect_node(browser, "Default", node, out); }
        }
    }
}

fn collect_node(browser: &str, profile: &str, node: ChromeNode, out: &mut Vec<BookmarkItem>) {
    match node {
        ChromeNode::Url { name, url } => {
            if url.starts_with("http") || url.starts_with("file:") {
                out.push(BookmarkItem { title: name, url, browser: browser.to_string(), profile: profile.to_string() });
            }
        }
        ChromeNode::Folder { name: _, children } => {
            if let Some(children) = children {
                for c in children { collect_node(browser, profile, c, out); }
            }
        }
    }
}

#[cfg(windows)]
fn chrome_paths() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|p| PathBuf::from(p).join("Google/Chrome/User Data"))
}
#[cfg(not(windows))]
fn chrome_paths() -> Option<PathBuf> { None }

#[cfg(windows)]
fn edge_paths() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|p| PathBuf::from(p).join("Microsoft/Edge/User Data"))
}
#[cfg(not(windows))]
fn edge_paths() -> Option<PathBuf> { None }

#[cfg(windows)]
fn opera_paths() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("Opera Software/Opera Stable"))
}
#[cfg(not(windows))]
fn opera_paths() -> Option<PathBuf> { None }

#[cfg(windows)]
fn opera_gx_paths() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("Opera Software/Opera GX Stable"))
}
#[cfg(not(windows))]
fn opera_gx_paths() -> Option<PathBuf> { None }

#[cfg(windows)]
fn all_bookmark_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Some(p) = chrome_paths() {
        files.push(p.join("Default/Bookmarks"));
    }
    if let Some(p) = edge_paths() {
        files.push(p.join("Default/Bookmarks"));
    }
    if let Some(p) = opera_paths() { files.push(p.join("Bookmarks")); }
    if let Some(p) = opera_gx_paths() { files.push(p.join("Bookmarks")); }
    files.into_iter().filter(|p| p.exists()).collect()
}
