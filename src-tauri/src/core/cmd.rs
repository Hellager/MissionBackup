use tauri::{ command, AppHandle, Manager, State, Window };
use serde::{ Serialize, Deserialize };
use log::{ debug, info, warn, error };
use crate::core::state::{ HandlerStatus, MissionHandlerState };
use crate::db::{ Record, mission::Mission };
use chrono::NaiveDateTime;

#[derive(Clone, Serialize, Deserialize)]
pub struct Response<T> {
    pub code: i32,
    pub data: T,
    pub msg: String,
}

impl<T> Response<T> {
    pub fn success(value: T) -> Response<T> {
        Response {
            code: 200,
            data: value,
            msg: "".to_string(),
        }
    }

    pub fn error(err_code: i32, err_msg: String) -> Response<bool> {
        Response {
            code: err_code,
            data: false,
            msg: err_msg
        }
    }
}

#[command]
pub async fn init_app(window: Window, state: State<'_, MissionHandlerState>) -> Result<Response<HandlerStatus>, Response<bool>> {    
    let mut guard = state.0.lock().await;
    if !guard.is_set {
        if let Err(error) = guard.initialize().await {
            error!("Failed to initialize state, errMsg: {:?}", error);
            return Err(Response::<bool>::error(500, format!("{:?}", error)));
        }     

        // close splashscreen
        if let Some(splashscreen) = window.get_window("splashscreen") {
            if let Err(error) = splashscreen.close() {
                error!("failed to init app, errMsg: {:?}", error);
                return Err(Response::<bool>::error(500, format!("{:?}", error)));
            }
        } else {
            error!("missing splashsceen window");
            return Err(Response::<bool>::error(404, format!("missing splashsceen window")));
        }        
    }

    // Show main window
    if let Some(main_window) = window.get_window("main") {
        if let Err(error) = main_window.show() {
            error!("failed to show main window, errMsg: {:?}", error);
            return Err(Response::<bool>::error(500, format!("{:?}", error)));
        }
    } else {
        error!("missing main window");
        return Err(Response::<bool>::error(404, format!("missing main window")));
    }
    
    let status = guard.status.clone();
    
    Ok(Response::success(status))
}

#[command]
pub async fn shutdown_app(app: AppHandle, state: State<'_, MissionHandlerState>) -> Result<Response<bool>, Response<bool>> {
    let mut guard = state.0.lock().await;

    match guard.shutdown() {
        Ok(()) => {
            app.exit(0);

            return Ok(Response::success(true));
        },
        Err(error) => {
            error!("Failed to shutdown, errMsg: {:?}", error);
            return Err(Response::<bool>::error(500, format!("{:?}", error)));
        }
    }
}

#[command]
pub async fn web_log(level: &str, msg: &str, state: State<'_, MissionHandlerState>) -> Result<Response<bool>, Response<bool>> {
    let mut guard = state.0.lock().await;
    
    if let Some(_) = &mut guard.log_handler {
        match level {
            "info" => {
                info!("{}", msg);
            },
            "warn" => {
                warn!("{}", msg);
            },
            "error" => {
                error!("{}", msg);
            },
            _ => {
                error!("Failed to log, errMsg: no match for level {}", level);
                return Err(Response::<bool>::error(400, format!("no match for level {}", level)));
            }
        }            
    }

    Ok(Response::success(true))
}

#[command]
pub fn show_item_in_explorer(path: &str) -> Result<Response<bool>, Response<bool>> {
    use crate::utils::explorer::show_in_explorer;

    match show_in_explorer(path) {
        Ok(()) => {
            return Ok(Response::success(true));
        },
        Err(error) => {
            error!("Failed to open path {}, errMsg: {:?}", path, error);
            return Err(Response::<bool>::error(500, format!("{:?}", error)));
        }
    }
}

#[command]
pub async fn sync_config(group: &str, config: crate::config::AppConfig, overwrite: bool, state: State<'_, MissionHandlerState>) -> Result<Response<crate::config::AppConfig>, Response<bool>> {
    let mut guard = state.0.lock().await;
    
    let cur = &mut guard.config;
    if overwrite {
        match group {
            "system" => {
                cur.system = config.system.clone();
            },
            "notify" => {
                cur.notify = config.notify.clone();
            },
            "watcher" => {
                cur.watcher = config.watcher.clone();
            },
            "screensaver" => {
                cur.screensaver = config.screensaver.clone();
            },
            _ => {
                error!("Failed to overwrite config, errMsg: no match for group {}", group);
                return Err(Response::<bool>::error(400, format!("no match for group {}", group)));
            }
        }    
    }
    debug!("sync config: {:?}, overwrite: {}", config, overwrite);

    return Ok(Response::success(cur.clone()));
}

#[command]
pub async fn create_record(table: &str, data: Record, state: State<'_, MissionHandlerState>) -> Result<Response<Record>, Response<bool>> {
    use crate::db::create_db_record;
    
    let mut guard = state.0.lock().await;

    if let Some(conn) = &mut guard.db_handler {
        let mut m_data = data.clone();
        match create_db_record(table, &mut m_data, conn) {
            Ok(val) => {
                debug!("create record: {:?}", val);
                return Ok(Response::success(val.clone()));
            },
            Err(error) => {
                error!("failed to create record, errMsg: {:?}", error);
                return Err(Response::<bool>::error(500, format!("{:?}", error)));
            }
        }
    }

    Err(Response::<bool>::error(503, "database unavailalbe".to_string()))
}

#[command]
pub async fn update_record(table: &str, data: Record, state: State<'_, MissionHandlerState>) -> Result<Response<Record>, Response<bool>> {
    use crate::db::update_db_record;
    
    let mut guard = state.0.lock().await;

    if let Some(conn) = &mut guard.db_handler {
        let mut m_data = data.clone();
        match update_db_record(table, &mut m_data, conn) {
            Ok(val) => {
                // debug!("update record: {:?}", val);
                return Ok(Response::success(val.clone()));
            },
            Err(error) => {
                error!("failed to update record, errMsg: {:?}", error);
                return Err(Response::<bool>::error(500, format!("{:?}", error)));
            }
        }
    }

    Err(Response::<bool>::error(503, "database unavailalbe".to_string()))
}

#[command]
pub async fn query_record(table: &str, uuid: Option<&str>, state: State<'_, MissionHandlerState>) -> Result<Response<Vec<Record>>, Response<bool>> {
    use crate::db::query_db_record;
    
    let mut guard = state.0.lock().await;

    if let Some(conn) = &mut guard.db_handler {
        match query_db_record(table, uuid, conn) {
            Ok(val) => {
                debug!("query table {:?}, {:?} records found", table, val.len());
                return Ok(Response::success(val));
            },
            Err(error) => {
                error!("failed to query record, errMsg: {:?}", error);
                return Err(Response::<bool>::error(500, format!("{:?}", error)));
            }
        }
    }

    Err(Response::<bool>::error(503, "database unavailalbe".to_string()))
}

#[command]
pub async fn delete_record(table: &str, uuid0: &str, uuid1: &str, state: State<'_, MissionHandlerState>) -> Result<Response<usize>, Response<bool>> {
    use crate::db::delete_db_record;
    
    let mut guard = state.0.lock().await;

    let uid0 = if uuid0.is_empty() { None } else { Some(uuid0) };
    let uid1 = if uuid1.is_empty() { None } else { Some(uuid1) };

    if let Some(conn) = &mut guard.db_handler {
        match delete_db_record(table, uid0, uid1, conn) {
            Ok(cnt) => {
                debug!("delete record {:?} or {:?} in table {:?} success", uuid0, uuid1, table);
                return Ok(Response::success(cnt));
            },
            Err(error) => {
                error!("failed to delete record: {:?} or {:?} in table {:?}, errMsg: {:?}", uuid0, uuid1, table, error);
                return Err(Response::<bool>::error(500, format!("{:?}", error)));
            }
        }
    }

    Err(Response::<bool>::error(503, "database unavailalbe".to_string()))
}

#[command]
pub async fn clear_record(table: &str, state: State<'_, MissionHandlerState>) -> Result<Response<usize>, Response<bool>> {
    use crate::db::clear_db_record;
    
    let mut guard = state.0.lock().await;

    if let Some(conn) = &mut guard.db_handler {
        match clear_db_record(table, conn) {
            Ok(cnt) => {
                debug!("clear record success, {:?} records removed, table {:?} is empty", cnt, table);
                return Ok(Response::success(cnt));
            },
            Err(error) => {
                error!("failed to clear table {:?} records, errMsg: {:?}", table, error);
                return Err(Response::<bool>::error(500, format!("{:?}", error)));
            }
        }
    }

    Err(Response::<bool>::error(503, "database unavailalbe".to_string()))
}

#[command]
pub async fn delete_backup(uuid: &str, state: State<'_, MissionHandlerState>) -> Result<Response<bool>, Response<bool>> {
    use crate::db::backup::delete_backup;

    let mut guard = state.0.lock().await;

    if let Some(conn) = &mut guard.db_handler {
        match delete_backup(uuid, conn) {
            Ok(_) => {
                debug!("delete backup {}", uuid);
                return Ok(Response::success(true));
            },
            Err(error) => {
                error!("failed to delete backup, errMsg: {:?}", error);
                return Err(Response::<bool>::error(500, format!("{:?}", error)));
            }
        }
    }

    Err(Response::<bool>::error(503, "database unavailalbe".to_string()))
}

#[command]
pub async fn set_mission_status(uuid: &str, stat: i16, state: State<'_, MissionHandlerState>) -> Result<Response<Mission>, Response<bool>> {    
    use crate::db::mission::update_mission_status;

    let mut guard = state.0.lock().await;

    if let Some(conn) = &mut guard.db_handler {
        match update_mission_status(conn, stat, uuid) {
            Ok(val) => {
                debug!("update mission {} status to {}", uuid, stat);
                return Ok(Response::success(val.clone()));
            },
            Err(error) => {
                error!("failed to update mission {} status, errMsg: {:?}", uuid, error);
                return Err(Response::<bool>::error(500, format!("{:?}", error)));
            }
        }
    }

    Err(Response::<bool>::error(503, "database unavailalbe".to_string()))
}

#[command]
pub async fn create_mission(mission: Mission, state: State<'_, MissionHandlerState>) -> Result<Response<bool>, Response<bool>> {    
    let mut guard = state.0.lock().await;

    match guard.create_job(&mission).await {
        Ok(val) => {
            debug!("create record: {:?}", val);
            return Ok(Response::success(val.clone()));
        },
        Err(error) => {
            error!("failed to create mission, errMsg: {:?}", error);
            return Err(Response::<bool>::error(500, format!("{:?}", error)));
        }
    }
}

#[command]
pub async fn delete_mission(uuid: &str, state: State<'_, MissionHandlerState>) -> Result<Response<bool>, Response<bool>> {
    let mut guard = state.0.lock().await;

    match guard.remove_job(uuid).await {
        Ok(val) => {
            debug!("remove job: {:?}", val);
            return Ok(Response::success(val.clone()));
        },
        Err(error) => {
            error!("failed to remove mission, errMsg: {:?}", error);
            return Err(Response::<bool>::error(500, format!("{:?}", error)));
        }
    }
}

#[command]
pub async fn query_statistic_record(mid: &str, start: Option<NaiveDateTime>, stop: Option<NaiveDateTime>, state: State<'_, MissionHandlerState>) -> Result<Response<Vec<crate::db::backup::Backup>>, Response<bool>> {
    use crate::db::backup::query_backup_record_with_date;
    
    let mut guard = state.0.lock().await;

    if let Some(conn) = &mut guard.db_handler {
        match query_backup_record_with_date(conn, mid, start.as_ref(), stop.as_ref()) {
            Ok(val) => {
                debug!("query statistic for mission {}, {:?} records found", mid, val.len());
                return Ok(Response::success(val));
            },
            Err(error) => {
                error!("failed to query record, errMsg: {:?}", error);
                return Err(Response::<bool>::error(500, format!("{:?}", error)));
            }
        }
    }

    Err(Response::<bool>::error(503, "database unavailalbe".to_string()))
}

#[command]
pub async fn query_db_info(state: State<'_, MissionHandlerState>) -> Result<Response<crate::db::utils::DBInfo>, Response<bool>> {
    use crate::db::utils::get_db_info;

    let mut guard = state.0.lock().await;

    if let Some(conn) = &mut guard.db_handler {
        match get_db_info(conn) {
            Ok(info) => {
                debug!("get db info {:?}", info);
                return Ok(Response::success(info.clone()));
            },
            Err(error) => {
                error!("failed to get db info, errMsg: {:?}", error);
                return Err(Response::<bool>::error(500, format!("{:?}", error)));
            }
        }
    }

    Err(Response::<bool>::error(503, "database unavailalbe".to_string()))
}

#[command]
pub async fn clean_database(state: State<'_, MissionHandlerState>) -> Result<Response<crate::db::utils::DBInfo>, Response<bool>> {
    use crate::db::utils::clean_database_records;

    let mut guard = state.0.lock().await;

    if let Some(conn) = &mut guard.db_handler {
        match clean_database_records(conn) {
            Ok(info) => {
                debug!("database cleaned, {:?} records removed", info.deleted);
                return Ok(Response::success(info.clone()));
            },
            Err(error) => {
                error!("failed to clean database, errMsg: {:?}", error);
                return Err(Response::<bool>::error(500, format!("{:?}", error)));
            }
        }
    }

    Err(Response::<bool>::error(503, "database unavailalbe".to_string()))
}

#[command]
pub async fn query_log_info(state: State<'_, MissionHandlerState>) -> Result<Response<crate::utils::logger::LogInfo>, Response<bool>> {
    use crate::utils::logger::{ LogInfo, get_log_info};
    let guard = state.0.lock().await;

    match &guard.log_handler {
        Some(log_path) => {
            if let Ok(info) = get_log_info(log_path) {
                return Ok(Response::success(info.clone()));
            }
        },
        None => {
            return Err(Response::<bool>::error(404, "log file not found".to_string()));
        }
    }

    #[cfg(debug_assertions)] {
        return Ok(Response::<LogInfo>::success(LogInfo {
            path: "term".to_string(),
            size: 0,
        }));        
    }


    #[cfg(not(debug_assertions))] {
        return Err(Response::<bool>::error(404, "log file not found".to_string()));        
    }
}

#[command]
pub async fn clean_app_log(state: State<'_, MissionHandlerState>) -> Result<Response<u64>, Response<bool>> {
    use crate::utils::logger::clean_log;
    let guard = state.0.lock().await;

    match &guard.log_handler {
        Some(log_path) => {
            if let Ok(size) = clean_log(log_path) {
                return Ok(Response::success(size));
            }
        },
        None => {
            return Err(Response::<bool>::error(404, "log file not found".to_string()));
        }
    }

    #[cfg(debug_assertions)] {
        return Ok(Response::<u64>::success(0));        
    }

    #[cfg(not(debug_assertions))] {
        return Err(Response::<bool>::error(404, "log file not found".to_string()));        
    }
}

#[command]
pub fn migrate_from_old(path: &str) -> Result<Response<crate::utils::migrate::MigratedData>, Response<bool>> {
  use crate::utils::migrate::parse_data_file;

  match parse_data_file(path) {
    Ok(data) => {
        return Ok(Response::success(data));
    },
    Err(error) => {
        error!("failed to migrate from {}", path);
        return Err(Response::<bool>::error(500, format!("failed to migrate from {}, errMsg: {:?}", path, error)));
    }
  }
}
