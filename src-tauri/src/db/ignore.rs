use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use chrono::{NaiveDateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use super::procedure::Procedure;
use crate::utils::common::rand_number;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = super::schema::ignore)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Ignore {
    pub id: i32,
    pub ignore_id: String,
    pub procedure_id: String,
    pub keyword: String,
    pub reserved_0: String,
    pub reserved_1: String,
    pub reserved_2: String,
    pub create_at: NaiveDateTime,
    pub update_at: NaiveDateTime,
    pub is_deleted: i16,
    pub delete_at: NaiveDateTime,
}

impl Default for Ignore {
    fn default() -> Self {
        Ignore {
            id: rand_number(),
            ignore_id: Uuid::new_v4().to_string(),
            procedure_id: Uuid::new_v4().to_string(),
            keyword: "".to_string(),
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

pub fn create_ignore_record(
    conn: &mut SqliteConnection,
    data: &mut Ignore,
    procedure: &Procedure
) -> Result<Ignore, diesel::result::Error> {
    use super::schema::ignore::dsl::*;

    let cur_time: NaiveDateTime = Utc::now().naive_utc();
    data.id = ignore.count().get_result(conn).unwrap_or(0) as i32 + 1;
    data.ignore_id = Uuid::new_v4().to_string();
    data.procedure_id = procedure.procedure_id.clone();
    data.create_at = cur_time;
    data.update_at = cur_time;

    diesel::insert_into(ignore)
        .values(data.clone())
        .returning(Ignore::as_returning())
        .get_result(conn)
}

pub fn update_ignore_record(
    conn: &mut SqliteConnection,
    data: &mut Ignore,
) -> Result<Ignore, diesel::result::Error> {
    use super::schema::{ignore, ignore::ignore_id};

    let cur_time: NaiveDateTime = Utc::now().naive_utc();
    data.update_at = cur_time;

    diesel::update(ignore::table)
        .filter(ignore_id.eq(&data.ignore_id))
        .set(data.clone())
        .returning(Ignore::as_returning())
        .get_result(conn)
}

pub fn query_ignore_record(
    conn: &mut SqliteConnection,
    uuid: Option<&str>,
) -> Result<Vec<Ignore>, diesel::result::Error> {
    use super::schema::ignore::dsl::*;

    match uuid {
        Some(uid) => {
            ignore.filter(procedure_id.eq(uid))
            .filter(is_deleted.eq(0))
            .select(Ignore::as_select())
            .load(conn)  
        },
        None => {
            ignore.select(Ignore::as_select())
                    .filter(is_deleted.eq(0))
                    .load(conn) 
        }
    }
}

pub fn delete_ignore_record(
    conn: &mut SqliteConnection,
    nid: Option<&str>,
    pid: Option<&str>,
) -> Result<usize, diesel::result::Error> {
    use super::schema::ignore::dsl::*;

    if let Some(uuid) = nid {
        return diesel::update(ignore)
                    .filter(ignore_id.eq(uuid))
                    .set((
                        is_deleted.eq(1),
                        delete_at.eq(Utc::now().naive_utc())
                    ))
                    .execute(conn);      
    } else if let Some(uuid) = pid {
        return diesel::update(ignore)
                    .filter(procedure_id.eq(uuid))
                    .set((
                        is_deleted.eq(1),
                        delete_at.eq(Utc::now().naive_utc())
                    ))
                    .execute(conn);    
    }  

    Err(diesel::result::Error::NotFound)
}

pub fn clear_ignore_record(
    conn: &mut SqliteConnection, 
) -> Result<usize, diesel::result::Error> {
    use super::schema::ignore::dsl::*;

    diesel::delete(ignore)
        .execute(conn)
}

pub fn clean_record(
    conn: &mut SqliteConnection, 
) -> Result<usize, diesel::result::Error> {
    use super::schema::ignore::dsl::*;

    let cleaned: usize = diesel::delete(ignore.filter(is_deleted.eq(1))).execute(conn)?;

    let mut remaining: Vec<Ignore> = ignore.select(Ignore::as_select()).load(conn)?;
    for (idx, item) in remaining.iter_mut().enumerate() {
        let new_id = (idx + 1) as i32;
        diesel::update(ignore)
            .filter(ignore_id.eq(&item.ignore_id))
            .set(id.eq(new_id))
            .execute(conn)?;
    }

    Ok(cleaned)   
}

pub fn get_procedure_ignores(pid: &str, conn: &mut SqliteConnection) -> Vec<String> {
    let mut data = Vec::new();
    if let Ok(ignores) = query_ignore_record(conn, Some(pid)) {
        for ignore in ignores.iter() {
            if ignore.is_deleted == 1 {
                continue;
            }
            data.push(ignore.keyword.clone());
        }
    }   

    data
}
