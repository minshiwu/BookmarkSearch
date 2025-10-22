use serde::Serialize;
use crate::bookmarks::BookmarkItem;
use parking_lot::RwLock;

#[derive(Debug, Clone, Serialize)]
pub struct SearchResultItem {
    pub title: String,
    pub url: String,
    pub browser: String,
    pub profile: String,
    pub score: i32,
}

#[derive(Debug)]
pub struct SearchIndex {
    items: Vec<IndexedItem>,
}

#[derive(Debug, Clone)]
struct IndexedItem {
    item: BookmarkItem,
    title_lower: String,
    url_lower: String,
    title_pinyin_full: String,
    title_pinyin_initials: String,
}

impl SearchIndex {
    pub fn new(items: Vec<BookmarkItem>) -> Self {
        let items = items.into_iter().map(IndexedItem::from).collect();
        Self { items }
    }

    pub fn query(&self, q: &str, limit: usize) -> Vec<SearchResultItem> {
        let ql = q.trim().to_lowercase();
        if ql.is_empty() {
            return Vec::new();
        }
        let mut results: Vec<SearchResultItem> = self.items.iter().filter_map(|it| {
            let mut score = 0i32;
            if it.title_lower.contains(&ql) { score += 120; }
            if it.url_lower.contains(&ql) { score += 80; }
            if !it.title_pinyin_full.is_empty() && it.title_pinyin_full.contains(&ql) { score += 70; }
            if !it.title_pinyin_initials.is_empty() && it.title_pinyin_initials.contains(&ql) { score += 60; }
            if score == 0 { return None; }
            Some(SearchResultItem {
                title: it.item.title.clone(),
                url: it.item.url.clone(),
                browser: it.item.browser.clone(),
                profile: it.item.profile.clone(),
                score,
            })
        }).collect();
        results.sort_by_key(|r| -r.score);
        results.truncate(limit);
        results
    }
}

impl From<BookmarkItem> for IndexedItem {
    fn from(item: BookmarkItem) -> Self {
        let title_lower = item.title.to_lowercase();
        let url_lower = item.url.to_lowercase();
        let (title_pinyin_full, title_pinyin_initials) = to_pinyin(&item.title);
        Self { item, title_lower, url_lower, title_pinyin_full, title_pinyin_initials }
    }
}

fn to_pinyin(s: &str) -> (String, String) {
    #[cfg(feature = "pinyin")] {
        use pinyin::{ToPinyin, Pinyin};
        let mut full = String::new();
        let mut initials = String::new();
        for ch in s.chars() {
            if let Some(py) = ch.to_pinyin() {
                let p = py.plain().to_lowercase();
                initials.push(p.chars().next().unwrap_or('\0'));
                full.push_str(&p);
            } else if ch.is_ascii_alphanumeric() {
                let c = ch.to_ascii_lowercase();
                initials.push(c);
                full.push(c);
            }
        }
        (full, initials)
    }
    #[cfg(not(feature = "pinyin"))]
    {
        // fallback: just ascii
        let only_ascii: String = s.chars().filter(|c| c.is_ascii()).collect::<String>().to_lowercase();
        (only_ascii.clone(), only_ascii)
    }
}
