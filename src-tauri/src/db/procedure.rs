use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use chrono::{NaiveDateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::utils::common::rand_number;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = super::schema::procedure)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Procedure {
    pub id: i32,
    pub procedure_id: String,
    pub name: String,
    pub has_ignores: bool,

    /// Ignore method
    /// 
    /// `0` - no ignores
    /// 
    /// `1` - use custom ignores
    /// 
    /// `2` - use .gitignore 
    pub ignore_method: i16,

    /// Whether compress
    pub is_compress: bool,

    /// Compress format
    /// 
    /// `0` - no compress
    /// 
    /// `1` - zip
    /// 
    /// `2` - tar.gz
    /// 
    /// `3` - tar.bz2
    /// 
    /// `4` - tar.xz
    /// 
    /// `5` - 7z
    pub compress_format: i16,

    /// Which trigger backup
    /// 
    /// `0` - reserved
    /// 
    /// `1` - cron trigger
    /// 
    /// `2` - monitore trigger
    pub trigger: i16,

    /// Cron expression
    /// 
    /// sec   min   hour   day of month   month   day of week   year
    /// 
    /// *     *     *      *              *       *             * 
    pub cron_expression: String,

    /// Whether restrict backups
    /// 
    /// `0` - no restrict
    /// 
    /// `1` - days restrict
    /// 
    /// `2` - size restrict
    /// 
    /// `3` - days and size restrict
    pub restrict: i16,
    pub restrict_days: i16,
    pub restrict_size: i64, // in bytes

    pub reserved_0: String,
    pub reserved_1: String,
    pub reserved_2: String,
    pub create_at: NaiveDateTime,
    pub update_at: NaiveDateTime,
    pub is_deleted: i16,
    pub delete_at: NaiveDateTime,
}

impl Default for Procedure {
    fn default() -> Self {
        Procedure {
            id: rand_number(),
            procedure_id: Uuid::new_v4().to_string(),
            name: "".to_string(),
            has_ignores: false,
            ignore_method: 1,
            is_compress: false,
            compress_format: 1,
            trigger: 1,
            cron_expression: "".to_string(),
            restrict: 0,
            restrict_days: 3,
            restrict_size: 1024,
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

pub fn create_procedure_record(
    conn: &mut SqliteConnection,
    data: &mut Procedure
) -> Result<Procedure, diesel::result::Error> {
    use super::schema::procedure::dsl::*;

    let cur_time: NaiveDateTime = Utc::now().naive_utc();
    data.id = procedure.count().get_result(conn).unwrap_or(0) as i32 + 1;
    data.procedure_id = Uuid::new_v4().to_string();
    data.create_at = cur_time;
    data.update_at = cur_time;

    diesel::insert_into(procedure)
        .values(data.clone())
        .returning(Procedure::as_returning())
        .get_result(conn)
}

pub fn update_procedure_record(
    conn: &mut SqliteConnection,
    data: &mut Procedure,
) -> Result<Procedure, diesel::result::Error> {
    use super::schema::{procedure, procedure::procedure_id};

    let cur_time: NaiveDateTime = Utc::now().naive_utc();
    data.update_at = cur_time;

    diesel::update(procedure::table)
        .filter(procedure_id.eq(&data.procedure_id))
        .set(data.clone())
        .returning(Procedure::as_returning())
        .get_result(conn)
}

pub fn query_procedure_record(
    conn: &mut SqliteConnection,
    pid: Option<&str>,
) -> Result<Vec<Procedure>, diesel::result::Error> {
    use super::schema::procedure::dsl::*;

    match pid {
        Some(uid) => {
            procedure.filter(procedure_id.eq(uid))
                .filter(is_deleted.eq(0))
                .select(Procedure::as_select())
                .load(conn)
        },
        None => {
            procedure.select(Procedure::as_select())
                .filter(is_deleted.eq(0))
                .load(conn)
        }
    }
}

pub fn delete_procedure_record(
    conn: &mut SqliteConnection,
    pid: Option<&str>,
) -> Result<usize, diesel::result::Error> {
    use super::schema::procedure::dsl::*;
    use diesel::result::Error;

    match pid {
        Some(uid) => {
            return diesel::update(procedure)
                .filter(procedure_id.eq(uid))
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

pub fn clear_procedure_record(
    conn: &mut SqliteConnection, 
) -> Result<usize, diesel::result::Error> {
    use super::schema::procedure::dsl::*;

    diesel::delete(procedure)
        .execute(conn)
}

pub fn clean_record(
    conn: &mut SqliteConnection, 
) -> Result<usize, diesel::result::Error> {
    use super::schema::procedure::dsl::*;

    let cleaned: usize = diesel::delete(procedure.filter(is_deleted.eq(1))).execute(conn)?;

    let mut remaining: Vec<Procedure> = procedure.select(Procedure::as_select()).load(conn)?;
    for (idx, item) in remaining.iter_mut().enumerate() {
        let new_id = (idx + 1) as i32;
        diesel::update(procedure)
            .filter(procedure_id.eq(&item.procedure_id))
            .set(id.eq(new_id))
            .execute(conn)?;
    }

    Ok(cleaned)   
}
