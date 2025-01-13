use tauri::{plugin::TauriPlugin, Emitter, Runtime, Manager};

pub fn initialize_plugin_single_instance<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_single_instance::init(|app, _, _| {
        if let Some(main_window) = app.get_webview_window("main") {
            let _ = app.emit("another_instance", {});
            let _ = main_window.center();
            let _ = main_window.set_focus();
        }
    })
}