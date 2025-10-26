mod bookmarks;
mod index;
mod hotkey;
mod ui;


use parking_lot::RwLock;
use wry::application::event::{Event, StartCause, WindowEvent};
use wry::application::event_loop::{ControlFlow, EventLoopBuilder};
use wry::application::window::WindowBuilder;
use wry::webview::WebViewBuilder;

use crate::bookmarks::{scan_all_browsers, BookmarkItem};
use crate::index::SearchIndex;
use crate::ui::{HTML_INDEX, IpcMessage};

#[derive(Debug, Clone)]
enum AppEvent {
    Toggle,
    Search(String),
    OpenUrl(String),
}

struct AppState {
    index: RwLock<SearchIndex>,
}

fn main() -> wry::Result<()> {
    setup_logging();

    let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("Bookmark Launcher")
        .with_decorations(false)
        .with_always_on_top(true)
        .with_visible(false)
        .with_inner_size(wry::application::dpi::LogicalSize::new(900.0, 520.0))
        .build(&event_loop)
        .expect("failed to create window");

    // Initial index
    let initial_items: Vec<BookmarkItem> = scan_all_browsers();
    let state = AppState {
        index: RwLock::new(SearchIndex::new(initial_items)),
    };

    // Build webview
    let proxy_clone = proxy.clone();
    let webview = WebViewBuilder::new(window)?
        .with_url(&("data:text/html,".to_string() + &urlencoding::encode(HTML_INDEX)))?
        .with_ipc_handler(move |_window, msg: String| {
            if let Ok(ipc) = serde_json::from_str::<IpcMessage>(&msg) {
                match ipc {
                    IpcMessage::Search { query } => {
                        let _ = proxy_clone.send_event(AppEvent::Search(query));
                    }
                    IpcMessage::Open { url } => {
                        let _ = proxy_clone.send_event(AppEvent::OpenUrl(url));
                    }
                    IpcMessage::Hide => {
                        let _ = proxy_clone.send_event(AppEvent::Toggle);
                    }
                }
            }
        })
        .build()?;

    // Hotkey thread (Windows)
    hotkey::spawn_global_hotkey_listener(proxy.clone());

    // File watchers for bookmarks updates
    bookmarks::spawn_watchers(proxy.clone());

    // Event loop
    let mut last_search: Option<String> = None;

    event_loop.run(move |event, _target, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::NewEvents(StartCause::Init) => {
                // Center window on primary monitor
                if let Some(m) = webview.window().current_monitor() {
                    let size = webview.window().inner_size();
                    let msize = m.size();
                    let x = (msize.width.saturating_sub(size.width)) / 2;
                    let y = (msize.height.saturating_sub(size.height)) / 3;
                    use wry::application::dpi::{PhysicalPosition};
                    webview.window().set_outer_position(PhysicalPosition::new(x as i32, y as i32));
                }
            }
            Event::UserEvent(AppEvent::Toggle) => {
                let vis = webview.window().is_visible();
                if vis {
                    webview.window().set_visible(false);
                } else {
                    webview.window().set_visible(true);
                    webview.window().set_focus();
                    let _ = webview.evaluate_script("window.__focusInput && window.__focusInput();");
                }
            }
            Event::UserEvent(AppEvent::Search(query)) => {
                if last_search.as_ref() == Some(&query) { return; }
                last_search = Some(query.clone());
                let results = {
                    let idx = state.index.read();
                    idx.query(&query, 30)
                };
                let payload = serde_json::to_string(&results).unwrap_or("[]".to_string());
                let _ = webview.evaluate_script(&format!(
                    "window.__setResults && window.__setResults({});",
                    payload
                ));
            }
            Event::UserEvent(AppEvent::OpenUrl(url)) => {
                let _ = open::that(url);
                webview.window().set_visible(false);
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent { event: WindowEvent::Focused(f), .. } => {
                if !f {
                    // Hide when losing focus to behave like a launcher
                    webview.window().set_visible(false);
                }
            }
            _ => {}
        }
    })
}

fn setup_logging() {
    use simplelog::*;
    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());
}
