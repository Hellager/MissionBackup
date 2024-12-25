use tauri::{App, Manager};
use tauri_plugin_window_state::{WindowExt, StateFlags};
use crate::core::tray;
use crate::config::{AppConfig, load_app_config, save_app_config};
use log::{info, error, warn, debug};

pub fn setup_handler(app: &mut App) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let window = app.get_webview_window("main").unwrap();
    let state_flags: StateFlags = StateFlags::POSITION | StateFlags::VISIBLE;
    let _ = window.restore_state(state_flags);
    info!("restore main window state");

    let _ = tray::create_system_tray(app.handle());

    match load_app_config() {
        Ok(_) => {
            debug!("initialize app config success");
        }
        Err(error) => {
            error!("failed to initialize app config, errMsg: {:?}", error);
            warn!("use default config");
            let default_config = AppConfig::default();
            save_app_config(&default_config)?;
        }
    }

    Ok(())
}