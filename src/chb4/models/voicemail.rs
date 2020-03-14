use crate::schema::*;
use chrono::prelude::*;

#[derive(Queryable, Identifiable)]
#[table_name = "voicemails"]
pub struct Voicemail {
    pub id: i32,
    pub creator_id: i32,
    pub receiver_id: i32,
    pub created: NaiveDateTime,
    pub scheduled: Option<NaiveDateTime>,
    pub active: bool,
    pub message: String,
}

#[derive(Insertable)]
#[table_name = "voicemails"]
pub struct NewVoicemail {
    pub creator_id: i32,
    pub receiver_id: i32,
    pub created: NaiveDateTime,
    pub scheduled: Option<NaiveDateTime>,
    pub message: String,
}

#[derive(Identifiable, AsChangeset)]
#[table_name = "voicemails"]
pub struct SetActiveVoicemail {
    pub id: i32,
    pub active: bool,
}
