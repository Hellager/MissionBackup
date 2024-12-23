mod plugins;
mod utils;
mod core;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(crate::plugins::initialize_plugin_log())
        .plugin(crate::plugins::initialize_plugin_single_instance())
        .plugin(crate::plugins::initialize_plugin_autostart())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .setup(crate::core::setup::setup_handler)
        .on_window_event(core::window::on_window_event)
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
