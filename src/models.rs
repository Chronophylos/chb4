use super::schema::users;
use chrono::prelude::*;
use diesel::sql_types::{Datetime, Nullable, Unsigned, Varchar};

#[derive(Queryable)]
pub struct Bans {
    pub id: u32,
    pub user_id: u32,
    pub created: Datetime,
    pub until: Nullable<Datetime>,
}

#[derive(Queryable)]
pub struct Channels {
    pub id: u32,
    pub owner_id: u32,
    pub enabled: bool,
    pub paused: bool,
}

#[derive(Queryable)]
pub struct ChannelActionFilters {
    pub id: u32,
    pub channel_id: u32,
    pub action_name: Varchar,
}

#[derive(Queryable)]
pub struct Copypastas {
    pub id: u32,
    pub creator_id: u32,
    pub created: Datetime,
    pub name: Varchar,
    pub message: Varchar,
}

#[derive(Queryable)]
pub struct Persons {
    pub id: u32,
    pub first_name: Nullable<Varchar>,
    pub last_name: Nullable<Varchar>,
    pub dob: Nullable<Datetime>,
}

#[derive(Queryable)]
pub struct Users {
    pub id: u32,
    pub twitch_id: Nullable<Varchar>,
    pub name: Varchar,
    pub display_name: Nullable<Varchar>,
    pub first_seen: Nullable<Datetime>,
    pub last_seen: Nullable<Datetime>,
    pub person_id: Nullable<Unsigned<u8>>,
    pub permission: u8,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub twitch_id: &'a str,
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

#[derive(Queryable)]
pub struct UserSettings {
    pub id: u32,
    pub user_id: u32,
    pub birthdays: bool,
}

#[derive(Queryable)]
pub struct Voicemails {
    pub id: u32,
    pub creator_id: u32,
    pub channel_id: u32,
    pub receiver_id: u32,
    pub created: Datetime,
    pub scheduled: Nullable<Datetime>,
    pub active: bool,
    pub message: Varchar,
}
