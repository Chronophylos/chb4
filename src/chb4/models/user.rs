use super::channel::Channel;
use crate::schema::*;
use chrono::prelude::*;

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
#[belongs_to(Channel)]
#[belongs_to(UserSettings, foreign_key = "settings_id")]
#[table_name = "users"]
pub struct User {
    pub id: u32,
    pub twitch_id: Option<u64>,
    pub name: String,
    pub display_name: Option<String>,
    pub first_seen: Option<NaiveDateTime>,
    pub last_seen: Option<NaiveDateTime>,
    pub permission: u8,

    pub person_id: Option<u32>,
    pub channel_id: Option<u32>,
    pub settings_id: Option<u32>,
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
#[table_name = "user_settings"]
pub struct UserSettings {
    pub id: u32,
    pub birthdays: bool,
}
