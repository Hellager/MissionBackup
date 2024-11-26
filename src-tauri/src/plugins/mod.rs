use tauri::{AppHandle, Emitter, Manager};

pub fn on_another_instance(app: &AppHandle, _argv: Vec<String>, _cwd: String) {
    let windows = app.webview_windows();

    if let Some(_) = windows.get("main") {
        match app.emit("instance", _cwd) {
            Ok(()) => {
                println!("Prevent launching another instance");
            }
            Err(error) => {
                println!(
                    "Failed to send event about another instance, errMsg: {:?}",
                    error
                );
            }
        }
    }
}
