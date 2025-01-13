use tauri::{Manager, Window, WindowEvent};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

pub fn on_window_event(window: &Window, event: &WindowEvent) {
    match event {
        WindowEvent::CloseRequested { .. } => {
            if window.label() == "main" {
                let app_handle = window.app_handle();
                let state_flags: StateFlags = StateFlags::POSITION | StateFlags::VISIBLE;
                app_handle.save_window_state(state_flags).unwrap();
                app_handle.exit(0);
            }
        }
        _ => {}
    }
}
