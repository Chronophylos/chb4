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

#[derive(Queryable, Identifiable, Associations)]
#[belongs_to(User, foreign_key = "owner_id")]
#[table_name = "channels"]
pub struct Channel {
    pub id: u32,
    pub twitch_id: u64,
    pub owner_id: u32,
    pub enabled: bool,
    pub paused: bool,
}

#[derive(Queryable, Identifiable)]
#[table_name = "channel_action_filters"]
pub struct ChannelActionFilter {
    pub id: u32,
    pub channel_id: u32,
    pub action_name: String,
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
#[table_name = "people"]
pub struct Person {
    pub id: u32,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub dob: Option<NaiveDateTime>,
}

#[derive(Queryable, Identifiable, Associations)]
#[belongs_to(Person)]
#[table_name = "users"]
pub struct User {
    pub id: u32,
    pub twitch_id: Option<u64>,
    pub name: String,
    pub display_name: Option<String>,
    pub first_seen: Option<NaiveDateTime>,
    pub last_seen: Option<NaiveDateTime>,
    pub person_id: Option<u32>,
    pub permission: u8,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub twitch_id: u64,
    pub name: &'a str,
    pub display_name: &'a str,
    pub first_seen: &'a NaiveDateTime,
    pub last_seen: &'a NaiveDateTime,
}

#[derive(AsChangeset)]
#[table_name = "users"]
pub struct BumpUser<'a> {
    pub name: &'a str,
    pub display_name: &'a str,
    pub last_seen: &'a NaiveDateTime,
}

#[derive(Queryable, Identifiable, Associations)]
#[belongs_to(User)]
#[table_name = "user_settings"]
pub struct UserSettings {
    pub id: u32,
    pub user_id: u32,
    pub birthdays: bool,
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
