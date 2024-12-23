use tauri::{Manager, Window, WindowEvent};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

pub fn on_window_event(window: &Window, event: &WindowEvent) {
    match event {
        WindowEvent::CloseRequested { .. } => {
            let app = window.app_handle();
            let _ = app.save_window_state(StateFlags::all());
            log::debug!("save windows state");
        }
        _ => {}
    }
}
