use tauri::{Manager, Window, WindowEvent};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

pub fn on_window_event(window: &Window, event: &WindowEvent) {
    match event {
        WindowEvent::CloseRequested { .. } => {
            let app = window.app_handle();
            let state_flags: StateFlags = StateFlags::POSITION | StateFlags::VISIBLE;
            let _ = app.save_window_state(state_flags);
            log::debug!("save windows state");
        }
        _ => {}
    }
}
