use crate::models::User;
use crate::schema::*;
use chrono::prelude::*;

#[derive(Queryable, Identifiable, Associations)]
#[belongs_to(User)]
#[table_name = "bans"]
pub struct Ban {
    pub id: u32,
    pub user_id: u32,
    pub created: NaiveDateTime,
    pub until: Option<NaiveDateTime>,
}

#[derive(Queryable, Identifiable)]
#[table_name = "copypastas"]
pub struct Copypasta {
    pub id: u32,
    pub creator_id: u32,
    pub created: NaiveDateTime,
    pub name: String,
    pub message: String,
}

#[derive(Queryable, Identifiable)]
#[table_name = "voicemails"]
pub struct Voicemail {
    pub id: u32,
    pub creator_id: u32,
    pub channel_id: u32,
    pub receiver_id: u32,
    pub created: NaiveDateTime,
    pub scheduled: Option<NaiveDateTime>,
    pub active: bool,
    pub message: String,
}
