use wry::application::event_loop::EventLoopProxy;

use crate::AppEvent;

#[cfg(windows)]
pub fn spawn_global_hotkey_listener(proxy: EventLoopProxy<AppEvent>) {
    std::thread::spawn(move || unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{RegisterHotKey, MOD_ALT, WM_HOTKEY, MSG, GetMessageW};
        use windows::Win32::UI::Input::KeyboardAndMouse::VK_SPACE;
        use windows::Win32::Foundation::HWND;

        let id = 1i32;
        let ok = RegisterHotKey(HWND(0), id, MOD_ALT.0 as u32, VK_SPACE.0 as u32).as_bool();
        if !ok {
            return;
        }
        let mut msg = MSG::default();
        loop {
            let ret = GetMessageW(&mut msg, HWND(0), 0, 0).0;
            if ret <= 0 { break; }
            if msg.message == WM_HOTKEY { let _ = proxy.send_event(AppEvent::Toggle); }
        }
    });
}

#[cfg(not(windows))]
pub fn spawn_global_hotkey_listener(_proxy: EventLoopProxy<AppEvent>) {
    // no-op
}
