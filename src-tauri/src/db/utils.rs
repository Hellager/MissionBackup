use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Serialize, Deserialize};

/// Struct Record
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DBInfo {
    pub path: String,

    pub deleted: u64,
}

/// Get database path.
/// 
/// Will use env variable `DATABASE_URL` as default database path.
/// 
/// If `DATABASE_URL` not exists: 
/// 
/// In debug mode, will create a new database at app home directory.
/// 
/// In release mode, will create a new database at app project data directory.
pub fn get_db_path() -> Result<String, std::io::Error> {
    use std::env;
    use dotenvy::dotenv;
    use std::path::Path;
    use path_absolutize::*;
    use log::error;
    use crate::utils::explorer::create_all;
    #[cfg(debug_assertions)]
    use crate::utils::common::get_app_home_dir;
    #[cfg(not(debug_assertions))]
    use directories::ProjectDirs;

    dotenv().ok();

    let mut database_url = env::var("DATABASE_URL").unwrap_or("".to_string());
    if database_url.is_empty() {
        #[cfg(not(debug_assertions))]
        {
            let application = option_env!("CARGO_PKG_NAME").unwrap_or("mission-backup");
            if let Some(proj_dirs) = ProjectDirs::from(
                "dev",
                "",
                application
            ) {
                database_url = proj_dirs.data_dir().join("database").join("mission_backup.db3").display().to_string();
            }            
        }
        
        #[cfg(debug_assertions)]
        if let Ok(app_dir) = get_app_home_dir() {
            database_url = app_dir.join("local_test.db3").display().to_string();
        }
    }

    if !Path::new(&database_url).exists() {
        if let Err(error) = create_all(&database_url, "file") {
            error!("Failed to create database, errMsg: {error}");
        }
    }

    if let Ok(abs_res) = Path::new(&database_url).absolutize() {
        if let Some(abs_path) = abs_res.to_str() {
            database_url = abs_path.to_string();
        }
    }

    Ok(database_url)
}

pub fn get_db_deleted_count(conn: &mut SqliteConnection) -> Result<u64, std::io::Error> {
    use crate::db::schema::{
        backup::dsl::*,
        ignore::dsl::*,
        mission::dsl::*,
        procedure::dsl::*
    };

    let mut count: u64 = 0;
    count += backup.filter(super::schema::backup::is_deleted.eq(1)).count().get_result(conn).unwrap_or(0) as u64;
    count += ignore.filter(super::schema::ignore::is_deleted.eq(1)).count().get_result(conn).unwrap_or(0) as u64;
    count += mission.filter(super::schema::mission::is_deleted.eq(1)).count().get_result(conn).unwrap_or(0) as u64;
    count += procedure.filter(super::schema::procedure::is_deleted.eq(1)).count().get_result(conn).unwrap_or(0) as u64;  

    Ok(count)
}

pub fn get_db_info(conn: &mut SqliteConnection) -> Result<DBInfo, std::io::Error> {
    let db_path = get_db_path()?;
    let db_deleted = get_db_deleted_count(conn)?;

    Ok(DBInfo {
        path: db_path,
        deleted: db_deleted,
    })
}

pub fn clean_database_records(conn: &mut SqliteConnection) -> Result<DBInfo, std::io::Error> {
    use super::clean_db_record;
    use log::error;
    use std::io::{Error, ErrorKind};

    let db_path = get_db_path()?;

    let tables = vec!["backup", "ignore", "mission", "procedure"];
    let mut cleaned_cnt: usize = 0;
    for item in tables {
        match clean_db_record(item, conn) {
            Ok(cnt) => {
                cleaned_cnt += cnt;
            },
            Err(error) => {
                error!("failed to clean table {:?} record, errMsg: {:?}", item, error);
                return Err(Error::from(ErrorKind::Other));
            }
        }
    }

    Ok(DBInfo {
        path: db_path,
        deleted: cleaned_cnt as u64,
    })
}
