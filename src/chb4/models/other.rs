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

#[derive(Queryable, Identifiable)]
#[table_name = "voicemails"]
pub struct Voicemail {
    pub id: i32,
    pub creator_id: i32,
    pub channel_id: i32,
    pub receiver_id: i32,
    pub created: NaiveDateTime,
    pub scheduled: Option<NaiveDateTime>,
    pub active: bool,
    pub message: String,
}