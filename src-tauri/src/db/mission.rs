use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use chrono::{NaiveDateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::utils::common::rand_number;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = super::schema::mission)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Mission {
    pub id: i32,
    pub mission_id: String,
    pub procedure_id: String,
    pub name: String,

    /// Mission status
    /// 
    /// `0` - pause
    /// 
    /// `1` - running
    /// 
    /// `2` - backuping
    pub status: i16,
    pub description: String,

    /// Target source path(absolute)
    pub src_path: String,

    /// Target save path(absolute)
    pub dst_path: String,

    /// Target path type
    /// 
    /// `0` - unknown
    /// 
    /// `1` - file
    /// 
    /// `2` - directory
    pub path_type: i16,

    /// for cron job
    pub next_runtime: NaiveDateTime,

    /// for monitor job
    pub last_trigger: NaiveDateTime,

    pub reserved_0: String,
    pub reserved_1: String,
    pub reserved_2: String,
    pub create_at: NaiveDateTime,
    pub update_at: NaiveDateTime,
    pub is_deleted: i16,
    pub delete_at: NaiveDateTime,
}

impl Default for Mission {
    fn default() -> Self {
        Mission {
            id: rand_number(),
            mission_id: Uuid::new_v4().to_string(),
            procedure_id: Uuid::new_v4().to_string(),
            name: "".to_string(),
            status: 0,
            description: "".to_string(),
            src_path: "".to_string(),
            dst_path: "".to_string(),
            path_type: 0,
            next_runtime: Utc::now().naive_utc(),
            last_trigger: Utc::now().naive_utc(),
            reserved_0: "".to_string(),
            reserved_1: "".to_string(),
            reserved_2: "".to_string(),
            create_at: Utc::now().naive_utc(),
            update_at: Utc::now().naive_utc(),
            is_deleted: 0,
            delete_at: Utc::now().naive_utc(),
        }
    }
}

pub fn create_mission_record(
    conn: &mut SqliteConnection,
    data: &mut Mission
) -> Result<Mission, diesel::result::Error> {
    use super::schema::mission::dsl::*;

    let cur_time: NaiveDateTime = Utc::now().naive_utc();
    data.id = mission.count().get_result(conn).unwrap_or(0) as i32 + 1;
    data.mission_id = Uuid::new_v4().to_string();
    data.create_at = cur_time;
    data.update_at = cur_time;

    diesel::insert_into(mission)
        .values(data.clone())
        .returning(Mission::as_returning())
        .get_result(conn)
}

pub fn update_mission_record(
    conn: &mut SqliteConnection,
    data: &mut Mission,
) -> Result<Mission, diesel::result::Error> {
    use super::schema::{mission, mission::mission_id};

    let cur_time: NaiveDateTime = Utc::now().naive_utc();
    data.update_at = cur_time;

    diesel::update(mission::table)
        .filter(mission_id.eq(&data.mission_id))
        .set(data.clone())
        .returning(Mission::as_returning())
        .get_result(conn)
}

pub fn update_mission_status(
    conn: &mut SqliteConnection,
    stat: i16,
    mid: &str,
) -> Result<Mission, diesel::result::Error> {
    use super::schema::mission::dsl::*;

    diesel::update(mission)
        .filter(mission_id.eq(mid))
        .set(status.eq(stat))
        .returning(Mission::as_returning())
        .get_result(conn)
}

pub fn update_mission_time(
    conn: &mut SqliteConnection,
    label: &str,
    time: &chrono::DateTime<chrono::Utc>,
    mid: &str,
) -> Result<Mission, diesel::result::Error> {
    use super::schema::mission::dsl::*;

    let runtime = time.naive_utc();

    match label {
        "next" => {
            diesel::update(mission)
                .filter(mission_id.eq(mid))
                .set(next_runtime.eq(runtime))
                .returning(Mission::as_returning())
                .get_result(conn)
        },
        "last" => {
            diesel::update(mission)
                .filter(mission_id.eq(mid))
                .set(last_trigger.eq(runtime))
                .returning(Mission::as_returning())
                .get_result(conn)
        },
        _ => {
            return Err(diesel::result::Error::NotFound);
        }
    }
}

pub fn query_mission_record(
    conn: &mut SqliteConnection,
    mid: Option<&str>,
) -> Result<Vec<Mission>, diesel::result::Error> {
    use super::schema::mission::dsl::*;

    match mid {
        Some(uid) => {
            mission.filter(mission_id.eq(uid))
                .filter(is_deleted.eq(0))
                .select(Mission::as_select())
                .load(conn)
        },
        None => {
            mission.select(Mission::as_select())
                .filter(is_deleted.eq(0))
                .load(conn)
        }
    }
}

pub fn delete_mission_record(
    conn: &mut SqliteConnection,
    mid: Option<&str>,
) -> Result<usize, diesel::result::Error> {
    use super::schema::mission::dsl::*;
    use diesel::result::Error;

    match mid {
        Some(uid) => {
            return diesel::update(mission)
                .filter(mission_id.eq(uid))
                .set((
                    is_deleted.eq(1),
                    delete_at.eq(Utc::now().naive_utc())
                ))
                .execute(conn); 
        },
        None => {
            return Err(Error::NotFound);
        }
    }
}

pub fn clear_mission_record(
    conn: &mut SqliteConnection, 
) -> Result<usize, diesel::result::Error> {
    use super::schema::mission::dsl::*;

    diesel::delete(mission)
        .execute(conn)
}

pub fn get_mission_related_record(mid: &str, conn: &mut SqliteConnection) -> Result<crate::db::Record, std::io::Error> {
    use super::{Record, procedure::query_procedure_record};
    use std::io::{ Error, ErrorKind };

    let mut record = Record::default();
    if let Ok(missions) = query_mission_record(conn, Some(mid)) {
        if missions.len() == 0 {
            return Err(Error::from(ErrorKind::NotFound));
        }

        let mission = &missions[0];
        record.mission = missions[0].clone();
        if let Ok(procedures) = query_procedure_record(conn, Some(&mission.procedure_id)) {
            if procedures.len() == 0 {
                return Err(Error::from(ErrorKind::NotFound));
            }

            record.procedure = procedures[0].clone();
            if record.procedure.is_deleted == 1 {
                return Err(Error::from(ErrorKind::NotFound));
            }
        }

        return Ok(record);
    }   

    Err(Error::from(ErrorKind::NotFound))
}

pub fn clean_record(
    conn: &mut SqliteConnection, 
) -> Result<usize, diesel::result::Error> {
    use super::schema::mission::dsl::*;

    let cleaned: usize = diesel::delete(mission.filter(is_deleted.eq(1))).execute(conn)?;

    let mut remaining: Vec<Mission> = mission.select(Mission::as_select()).load(conn)?;
    for (idx, item) in remaining.iter_mut().enumerate() {
        let new_id = (idx + 1) as i32;
        diesel::update(mission)
            .filter(mission_id.eq(&item.mission_id))
            .set(id.eq(new_id))
            .execute(conn)?;
    }

    Ok(cleaned)   
}
