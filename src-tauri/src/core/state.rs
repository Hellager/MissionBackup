use tokio::{runtime::Handle, sync::mpsc::Receiver, sync::Mutex};
use std::io::{Error, ErrorKind};
use crate::utils::logger::initialize_logger;
use log::{debug, error, info, warn};
use serde::{Serialize, Deserialize};
use crate::config::AppConfig;
use tauri::{ AppHandle, Manager };
use diesel::sqlite::SqliteConnection;
use tokio_cron_scheduler::JobScheduler;
use notify::{Error as NotifyError, ReadDirectoryChangesWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{
    new_debouncer, DebounceEventResult, DebouncedEvent, Debouncer, FileIdMap,
};
use std::{collections::HashMap, path::Path, time::Duration};
use crate::db::{
    mission::Mission,
    procedure::Procedure,
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerStatus {
    pub log: bool,
    pub app: bool,
    pub config: bool,
    pub database: bool,
    pub cron: bool,
    pub watcher: bool,
}

impl Default for HandlerStatus {
    fn default() -> Self {
        HandlerStatus {
            log: false,
            app: false,
            config: false,
            database: false,
            cron: false,
            watcher: false
        }
    }
}

pub struct MissionHandler {
    pub is_set: bool,
    pub status: HandlerStatus,
    pub config: AppConfig,
    pub log_handler: Option<String>,
    pub app_handler: Option<AppHandle>,
    pub db_handler: Option<SqliteConnection>,
    pub cron_handler: Option<JobScheduler>,
    pub watcher_handler: Option<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>>,
    pub watcher_receiver: Option<Receiver<Result<Vec<DebouncedEvent>, Vec<NotifyError>>>>,

    /// Cron jobs
    /// Mission id : Job ib
    pub cron_jobs: HashMap<String, Uuid>,

    /// Monitor jobs
    /// Mission id : Watch path
    pub monitor_jobs: HashMap<String, String>
}

impl MissionHandler {
    fn init_app_status(&mut self) -> Result<(), std::io::Error> {
        self.status = HandlerStatus::default();
        Ok(())
    }

    fn init_app_config(&mut self) -> Result<(), std::io::Error> {
        use crate::config::{ load_app_config, save_app_config };

        match load_app_config() {
            Ok(app_config) => {
                self.config = app_config;
                self.status.config = true;
                debug!("initialize app config success");
                return Ok(());
            },
            Err(error) => {
                error!("failed to initialize app config, errMsg: {:?}", error);
                warn!("use default config");
                self.config = AppConfig::default();
                self.status.config = true;
                save_app_config(&self.config)?;
                return Ok(());
            }
        }   
    }

    fn init_logger_handler(&mut self) -> Result<(), std::io::Error> {
        if let None = self.log_handler {
            match initialize_logger(None, None) {
                Ok(log_file) => {
                    self.log_handler = Some(log_file.clone());
                    self.status.log = true;
                    debug!("initialize logger handler success, log at {}", log_file);
                    return Ok(());
                },
                Err(error) => {
                    self.log_handler = None;
                    self.status.log = false;
                    println!("failed to initialize logger handler, errMsg: {:?}", error);
                    return Err(error);
                }
            }            
        } else {
            warn!("logger handler already exists");
            self.status.log = true;
            return Ok(());
        }
    }

    fn init_app_handler(&mut self) -> Result<(), std::io::Error> {
        if let None = self.app_handler {
            self.status.app = false;
            error!("failed to initialize app handler");
        } else {
            self.status.app = true;
            info!("app handler already exists");
        }

        Ok(())
    }

    pub fn init_db_handler(&mut self) -> Result<(), std::io::Error> {
        use crate::db::{establish_sqlite_connection, init_database};

        if let None = self.db_handler {
            match establish_sqlite_connection() {
                Ok(conn) => {
                    self.db_handler = Some(conn);
                    self.status.database = true;

                    if let Some(mut dconn) = self.db_handler.as_mut() {
                        init_database(&mut dconn)?;
                    }
                    
                    debug!("initialize db handler success");
                    return Ok(());
                },
                Err(error) => {
                    self.status.database = false;
                    error!("failed to initialize db handler, errMsg: {:?}", error);
                    return Err(Error::from(ErrorKind::Other));
                }
            }            
        } else {
            self.status.database = true;
            warn!("database handler already success");
            return Ok(());         
        }
    }

    async fn init_cron_handler(&mut self) -> Result<(), std::io::Error> {
        if let None = self.cron_handler {
            if let Ok(handler) = JobScheduler::new().await {
                if let Err(error) = handler.start().await {
                    error!("Failed to initialize cron handler, errMsg: {:?}", error);
                    self.status.cron = false;
                    return Err(Error::from(ErrorKind::Other));
                } else {
                    self.status.cron = true;
                    self.cron_handler = Some(handler);
                    debug!("Initialize cron handler success");
                }
            }          
        } else {
            warn!("Cron handler already success");
            self.status.cron = true;
        }

        Ok(())
    }

    async fn init_watcher_handler(&mut self) -> Result<(), std::io::Error> {
        if let None = self.watcher_handler {
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            let rt = Handle::current();

            let timeout = self.config.watcher.timeout;

            let debouncer = new_debouncer(
                Duration::from_secs(timeout),
                None,
                move |result: DebounceEventResult| {
                    let tx = tx.clone();

                    println!("calling by notify -> {:?}", &result);
                    rt.spawn(async move {
                        if let Err(e) = tx.send(result).await {
                            println!("Error sending event result: {:?}", e);
                        }
                    });
                },
            );

            match debouncer {
                Ok(watcher) => {
                    self.watcher_handler = Some(watcher);
                    self.watcher_receiver = Some(rx);
                    self.status.watcher = true;
                    debug!("Initialize notify handler success");
                    return Ok(());
                }
                Err(error) => {
                    self.status.watcher = false;
                    error!("Failed to initialize notify handler, errMsg: {:?}", error);

                    return Err(Error::from(ErrorKind::Other));
                }
            }                    
        }else {
            self.status.watcher = true;
            warn!("Notify handler already success");
            return Ok(());
        }
    }

    pub async fn initialize(&mut self) -> Result<(), std::io::Error> {
        self.init_app_status()?;        
        self.init_logger_handler()?;
        self.init_app_config()?;
        self.init_app_handler()?;
        self.init_db_handler()?;
        self.init_cron_handler().await?;
        self.init_watcher_handler().await?;

        self.is_set = true;
        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<(), std::io::Error> {      
        use crate::config::save_app_config;

        save_app_config(&self.config)?;

        Ok(())
    }

    pub async fn watch(&mut self, path: &str, mission_id: &str) -> Result<(), std::io::Error> {
        use crate::db::{establish_sqlite_connection, backup::create_backup};
        use super::cmd::Response;
        use std::path::Path;

        let watch_path = Path::new(path);
        if !watch_path.exists() {
            return Err(Error::from(ErrorKind::NotFound));
        }

        if let Some(watcher) = self.watcher_handler.as_mut() {
            if let Ok(()) = watcher.watcher().watch(watch_path, RecursiveMode::Recursive) {
                watcher
                    .cache()
                    .add_root(watch_path, RecursiveMode::Recursive);

                if let Some(mut rx) = self.watcher_receiver.take() {
                    let callback_id = mission_id.to_string().clone();
                    let callback_app = self.app_handler.clone();
                    tokio::spawn(async move {
                        while let Some(res) = rx.recv().await {
                            match res {
                                Ok(_events) => {
                                    if let Some(app) = &callback_app {
                                        if let Ok(mut conn) = establish_sqlite_connection() {
                                            match create_backup(&callback_id, &mut conn) {
                                                Ok(backup) => {
                                                    let _ = app.emit_all("backup", Response::success(backup));
                                                },
                                                Err(error) => {
                                                    let _ = app.emit_all("backup", Response::<bool>::error(500, format!("{:?}", error)));
                                                }
                                            }
                                        } else {
                                            let _ = app.emit_all("backup", Response::<bool>::error(500, "Failed connect to database".to_string()));
                                        }
                                    } else {
                                        error!("Invalid app instance when create backup");
                                    }
                                }
                                Err(errors) => {
                                    error!("Mission {} watch failed, errMsg: {:?}", callback_id, errors);
                                }
                            }
                        }
                    });

                    return Ok(());
                }
            }
            return Err(Error::from(ErrorKind::Unsupported));
        }

        Ok(())
    }

    pub async fn create_job(&mut self, mission: &Mission) -> Result<bool, std::io::Error> {
        use crate::db::query_db_record;

        if let Some(conn) = &mut self.db_handler {
            if let Ok(records) = query_db_record("procedure", Some(mission.procedure_id.as_str()), conn) {
                if records.len() == 0 {
                    return Err(Error::from(ErrorKind::Unsupported));
                }

                let config = &records[0].procedure;
                match config.trigger {
                    1 => {
                        self.create_cron_job(mission, config).await?;
                    },
                    2 => {
                        self.create_monitor_job(mission).await?;
                    },
                    _ => {
                        return Err(Error::from(ErrorKind::Unsupported));
                    }
                }
            }
        }

        Ok(true)
    }

    async fn create_cron_job(&mut self, mission: &Mission, procedure: &Procedure) -> Result<bool, std::io::Error> {
        use tokio_cron_scheduler::Job;
        use crate::db::{establish_sqlite_connection, mission::update_mission_time, backup::create_backup};
        use super::cmd::Response;

        let callback_id = mission.clone().mission_id;
        let callback_app = self.app_handler.clone();
        let create_res = Job::new(procedure.cron_expression.as_str(), move |_uuid, _l| {
            if let Some(app) = &callback_app {
                if let Ok(mut conn) = establish_sqlite_connection() {
                    match create_backup(&callback_id, &mut conn) {
                        Ok(backup) => {
                            let _ = app.emit_all("backup", Response::success(backup));
                        },
                        Err(error) => {
                            let _ = app.emit_all("backup", Response::<bool>::error(500, format!("{:?}", error)));
                        }
                    }
                } else {
                    let _ = app.emit_all("backup", Response::<bool>::error(500, "Failed connect to database".to_string()));
                }
            } else {
                error!("Invalid app instance when create backup");
            }
        });     

        match create_res {
            Ok(cron_job) => {
                if let Some(handler) = &mut self.cron_handler {
                    if let Ok(job_id) = handler.add(cron_job).await {
                        self.cron_jobs.insert(mission.mission_id.clone(), job_id);

                        if let Ok(Some(next_tick)) = handler.next_tick_for_job(job_id).await {
                            if let Some(conn) = &mut self.db_handler {
                                if let Err(error) = update_mission_time(conn, "next", &next_tick, &mission.mission_id) {
                                    error!("Failed to update cron job for time for mission {}, errMsg: {:?}", mission.name, error);
                                    return Err(Error::from(ErrorKind::Interrupted));
                                }                                
                            }
                        }                                
                    }
                }
            },
            Err(error) => {
                error!("Failed to create cron job for mission {}, errMsg: {:?}", mission.name, error);
                return Err(Error::from(ErrorKind::Interrupted));
            }
        }           

        Ok(true)
    }

    async fn create_monitor_job(&mut self, mission: &Mission) -> Result<bool, std::io::Error> {
        use crate::db::query_db_record;
        
        if let Some(conn) = &mut self.db_handler {
            if let Ok(procedures) = query_db_record("procedure", Some(mission.procedure_id.as_str()), conn) {
                if procedures.len() == 0 {
                    return Err(Error::from(ErrorKind::Unsupported));
                }

                let create_res = self.watch(&mission.src_path.as_str(), &mission.mission_id.as_str()).await;
                match create_res {
                    Ok(()) => {
                        info!("create monitore job success for mission {}", mission.name);
                        self.monitor_jobs.insert(mission.mission_id.clone(), mission.src_path.clone());
                        return Ok(true);
                    },
                    Err(error) => {
                        error!("Failed to create monitor job for mission {}, errMsg: {:?}", mission.name, error);
                        return Err(Error::from(ErrorKind::Interrupted));
                    }
                }           
            } else {
                error!("Failed to create monitor job for mission {}, errMsg: failed to query procedure", mission.name);
                return Err(Error::from(ErrorKind::InvalidData));
            }
        } 

        Ok(true)
    }

    pub async fn remove_job(&mut self, mission_id: &str) -> Result<bool, std::io::Error> {
        use crate::db::query_db_record;

        if let Some(conn) = &mut self.db_handler {
            if let Ok(m_records) = query_db_record("mission", Some(mission_id), conn) {
                if m_records.len() == 0 {
                    return Err(Error::from(ErrorKind::Unsupported));
                }

                let procedure_id = &m_records[0].mission.procedure_id;
                if let Ok(records) = query_db_record("procedure", Some(procedure_id.as_str()), conn) {
                    if records.len() == 0 {
                        return Err(Error::from(ErrorKind::Unsupported));
                    }

                    let config = &records[0].procedure;
                    debug!("mission procedure: {:?}", config);
                    match config.trigger {
                        1 => { // Cron
                            self.remove_cron_job(mission_id).await?;
                        },
                        2 => {
                            self.remove_monitore_job(mission_id)?;
                        },
                        _ => {
                            debug!("may here?");
                            return Err(Error::from(ErrorKind::Unsupported));
                        }
                    }
                }                
            }
        }

        Ok(true)
    }

    async fn remove_cron_job(&mut self, mission_id: &str) -> Result<bool, std::io::Error> {
        if self.cron_jobs.contains_key(mission_id) {
            if let Some(job_id) = self.cron_jobs.get(mission_id) {
                if let Some(handler) = &mut self.cron_handler {
                    match handler.remove(job_id).await {
                        Ok(()) => {
                            debug!("remove mission {}", mission_id);
                            return Ok(true);
                        },
                        Err(error) => {
                            error!("Failed to remove cron job for mission {}, errMsg: {:?}", mission_id, error);
                            return Err(Error::from(ErrorKind::Other));
                        }
                    }
                }
            }
        }

        Ok(true)
    }

    fn remove_monitore_job(&mut self, mission_id: &str) -> Result<bool, std::io::Error> {
        if self.monitor_jobs.contains_key(mission_id) {
            if let Some(watch_path) = self.monitor_jobs.get(mission_id) {
                if let Some(handler) = &mut self.watcher_handler {
                    match handler.watcher().unwatch(Path::new(watch_path)) {
                        Ok(()) => {
                            debug!("remove mission {}", mission_id);
                            return Ok(true);
                        },
                        Err(error) => {
                            error!("Failed to remove cron job for mission {}, errMsg: {:?}", mission_id, error);
                            return Err(Error::from(ErrorKind::Other));
                        }
                    }
                }
            }
        }

        Ok(true)
    }
}

pub struct MissionHandlerState(pub Mutex<MissionHandler>);
