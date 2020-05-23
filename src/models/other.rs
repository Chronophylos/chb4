use crate::schema::*;
use chrono::prelude::*;

#[derive(Queryable, Identifiable)]
#[table_name = "copypastas"]
pub struct Copypasta {
    pub id: i32,
    pub creator_id: i32,
    pub created: NaiveDateTime,
    pub name: String,
    pub message: String,
}
