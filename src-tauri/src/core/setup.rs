use tauri::{App, Manager};
use std::collections::HashMap;

pub fn setup_handler(app: &mut App) -> Result<(), Box<dyn std::error::Error + 'static>> {
    use super::state::{ MissionHandler, HandlerStatus, MissionHandlerState };
    use super::window;
    use crate::config::AppConfig;
    use tokio::sync::Mutex;
    use log::error;

    if let Some(main_window) = app.get_window("main") {
        // add window shadows to app, not available on linux now
        #[cfg(not(target_os = "linux"))]
        window::init_window_shadow(&main_window, true);
    } else {
        error!("failed to init window shadow");
    }

    let state = MissionHandlerState(Mutex::new(MissionHandler {
        is_set: false,
        status: HandlerStatus::default(),
        config: AppConfig::default(),

        app_handler: Some(app.handle().clone()),
        log_handler: None,
        db_handler: None,
        cron_handler: None,
        watcher_handler: None,
        watcher_receiver: None,
        cron_jobs: HashMap::new(),
        monitor_jobs: HashMap::new()
    }));

    app.manage(state);

    Ok(())
}

pub fn setup_command() -> Box<dyn Fn(tauri::Invoke<tauri::Wry>) + Send + Sync> {
    use super::cmd::*;

    Box::new(tauri::generate_handler![
        init_app,
        shutdown_app,
        web_log,
        show_item_in_explorer,
        sync_config,
        create_record,
        update_record,
        query_record,
        delete_record,
        clear_record,
        delete_backup,
        set_mission_status,
        create_mission,
        delete_mission,
        query_statistic_record,
        query_db_info,
        clean_database,
        query_log_info,
        clean_app_log,
        migrate_from_old
    ])
}
