use super::channel::Channel;
use crate::schema::*;
use chrono::prelude::*;

#[derive(Queryable, Identifiable)]
#[table_name = "people"]
pub struct Person {
    pub id: i32,
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
    pub id: i32,
    pub twitch_id: Option<i64>,
    pub name: String,
    pub display_name: Option<String>,
    pub first_seen: Option<NaiveDateTime>,
    pub last_seen: Option<NaiveDateTime>,
    pub permission: i16,
    pub banned_until: Option<NaiveDateTime>,

    pub person_id: Option<i32>,
    pub channel_id: Option<i32>,
    pub settings_id: Option<i32>,
}

impl User {
    pub fn banned(&self, now: &DateTime<Local>) -> bool {
        match self.banned_until {
            None => false,
            Some(until) => now.naive_utc() < until,
        }
    }

    pub fn display_name_or_name(&self) -> String {
        self.display_name
            .clone()
            .or_else(|| Some(self.name.clone()))
            .unwrap()
    }
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub twitch_id: i64,
    pub name: &'a str,
    pub display_name: &'a str,
    pub first_seen: &'a NaiveDateTime,
    pub last_seen: &'a NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUserWithName<'a> {
    pub name: &'a str,
}

#[derive(AsChangeset)]
#[table_name = "users"]
pub struct FixUserWithOnlyName<'a> {
    pub twitch_id: i64,
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
    pub id: i32,
    pub birthdays: bool,
}
