use tauri::{App, Manager};
use tauri_plugin_window_state::{StateFlags, WindowExt};

pub fn setup_handler(app: &mut App) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let window = app.get_webview_window("main").unwrap();
    let state_flags: StateFlags = StateFlags::POSITION | StateFlags::VISIBLE;
    let _ = window.restore_state(state_flags);

    Ok(())
}
