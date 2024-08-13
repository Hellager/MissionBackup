pub mod ignore;
pub mod mission;
pub mod backup;
pub mod procedure;
pub mod schema;
pub mod utils;

use self::backup::Backup;
use self::ignore::Ignore;
use self::mission::Mission;
use self::procedure::Procedure;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Serialize, Deserialize};

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Record {
    pub backup: Backup,
    pub ignore: Ignore,
    pub mission: Mission,
    pub procedure: Procedure,
}

impl Default for Record {
    fn default() -> Self {
        Record {
            backup: Backup::default(),
            ignore: Ignore::default(),
            mission: Mission::default(),
            procedure: Procedure::default(),
        }
    }
}

pub fn establish_sqlite_connection() -> Result<SqliteConnection, diesel::result::ConnectionError> {
    use self::utils::get_db_path;

    match get_db_path() {
        Ok(database_url) => SqliteConnection::establish(&database_url),
        Err(_) => Err(diesel::result::ConnectionError::InvalidConnectionUrl("".to_string()))
    }
}

pub fn init_database(conn: &mut SqliteConnection) -> Result<(), std::io::Error> {
    use std::io::{ Error, ErrorKind };
    use log::error;

    match conn.run_pending_migrations(MIGRATIONS) {
        Ok(_) => {
            return Ok(())
        },
        Err(error) => {
            error!("Failed to migration database, errMsg: {:?}", error);
            return Err(Error::from(ErrorKind::Other));
        }
    }
}

pub fn create_db_record(table: &str, data: &mut Record, conn: &mut SqliteConnection) -> Result<Record, diesel::result::Error> {
    use diesel::result::Error;
    
    match table {
        "ignore" => {
            ignore::create_ignore_record(conn, &mut data.ignore, &data.procedure)?;
        },
        "procedure" => {
            procedure::create_procedure_record(conn, &mut data.procedure)?;     
        },
        "mission" => {
            mission::create_mission_record(conn, &mut data.mission)?;
        },
        "backup" => {
            backup::create_backup_record(conn, &mut data.backup, &data.mission)?;
        },
        _ => {
            return Err(Error::from(Error::NotFound));
        }
    }

    Ok(data.clone())
}

pub fn update_db_record(table: &str, data: &mut Record, conn: &mut SqliteConnection) -> Result<Record, diesel::result::Error> {
    use diesel::result::Error;
    
    match table {
        "ignore" => {
            ignore::update_ignore_record(conn, &mut data.ignore)?; 
        },
        "procedure" => {
            procedure::update_procedure_record(conn, &mut data.procedure)?;       
        },
        "mission" => {
            mission::update_mission_record(conn, &mut data.mission)?;
        },
        "backup" => {
            // backup::update_backup_record(conn, &mut data.backup)?;
        },
        _ => {
            return Err(Error::from(Error::NotFound));
        }
    }

    Ok(data.clone())    
}

pub fn query_db_record(table: &str, uid: Option<&str>, conn: &mut SqliteConnection) -> Result<Vec<Record>, diesel::result::Error> {
    use diesel::result::Error;
    let mut res: Vec<Record> = Vec::new();

    match table {
        "ignore" => {
            let records: Vec<Ignore> = ignore::query_ignore_record(conn, uid)?;

            for item in records {
                let mut full = Record::default();
                full.ignore = item;
                res.push(full);
            }
        },
        "procedure" => {
            let records: Vec<Procedure> = procedure::query_procedure_record(conn, uid)?;

            for item in &records {
                let mut full = Record::default();
                full.procedure = item.clone();
                res.push(full);
            }     
        },
        "mission" => {
            let records: Vec<Mission> = mission::query_mission_record(conn, uid)?;

            for item in records {
                let mut full = Record::default();
                full.mission = item;
                res.push(full);
            }
        },
        "backup" => {
            let records = backup::query_backup_record(conn, None, uid)?;

            for item in records {
                let mut full = Record::default();
                full.backup = item;
                res.push(full);
            }
        },
        _ => {
            return Err(Error::from(Error::NotFound));
        }
    }

    Ok(res)
}

pub fn delete_db_record(table: &str, uuid_0: Option<&str>, uuid_1: Option<&str>, conn: &mut SqliteConnection) -> Result<usize, diesel::result::Error> {
    use diesel::result::Error;
    let remove_cnt: usize;
    
    match table {
        "ignore" => {
            remove_cnt = ignore::delete_ignore_record(conn, uuid_0, uuid_1)?;
        },
        "procedure" => {
            remove_cnt = procedure::delete_procedure_record(conn, uuid_0)?;   
        },
        "mission" => {
            remove_cnt = mission::delete_mission_record(conn, uuid_0)?;
        },
        "backup" => {
            remove_cnt = backup::delete_backup_record(conn, uuid_0, uuid_1)?;
        },
        _ => {
            return Err(Error::from(Error::NotFound));
        }
    }

    Ok(remove_cnt)    
}

pub fn clear_db_record(table: &str, conn: &mut SqliteConnection) -> Result<usize, diesel::result::Error> {
    use diesel::result::Error;
    let remove_cnt: usize;
    
    match table {
        "ignore" => {
            remove_cnt = ignore::clear_ignore_record(conn)?;
        },
        "procedure" => {
            remove_cnt = procedure::clear_procedure_record(conn)?;       
        },
        "mission" => {
            remove_cnt = mission::clear_mission_record(conn)?;
        },
        "backup" => {
            remove_cnt = backup::clear_backup_record(conn)?;
        },
        _ => {
            return Err(Error::from(Error::NotFound));
        }
    }

    Ok(remove_cnt)    
}

pub fn clean_db_record(table: &str, conn: &mut SqliteConnection) -> Result<usize, diesel::result::Error> {
    use diesel::result::Error;
    let remove_cnt: usize;
    
    match table {
        "ignore" => {
            remove_cnt = ignore::clean_record(conn)?;
        },
        "procedure" => {
            remove_cnt = procedure::clean_record(conn)?;       
        },
        "mission" => {
            remove_cnt = mission::clean_record(conn)?;
        },
        "backup" => {
            remove_cnt = backup::clean_record(conn)?;
        },
        _ => {
            return Err(Error::from(Error::NotFound));
        }
    }

    Ok(remove_cnt)    
}
