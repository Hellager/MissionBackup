use tauri::{plugin::TauriPlugin, Emitter, Manager};

pub fn initialize_plugin_single_instance<R: tauri::Runtime>() -> TauriPlugin<R> {
    tauri_plugin_single_instance::init(|app, _, _| {
        // println!("app: {:?}, args: {:?}, cwd: {:?}", app, args, cwd);

        if let Some(main_window) = app.get_webview_window("main") {    
            let _ = app.emit("main", "another_instance");
            let _ = main_window.center();
            let _ = main_window.set_focus();
        }
    })
}