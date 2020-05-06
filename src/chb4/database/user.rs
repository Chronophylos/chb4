use super::{Channel, Connection, Voicemail};
use crate::schema::*;
use chrono::prelude::*;
use diesel::prelude::*;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Getting user (twitch_id: {}): {}", twitch_id, source))]
    GetUserByTwitchID {
        twitch_id: i64,
        source: diesel::result::Error,
    },

    #[snafu(display("Getting user (name: {}): {}", name, source))]
    GetUserByName {
        name: String,
        source: diesel::result::Error,
    },

    #[snafu(display("Getting user (id: {}): {}", id, source))]
    GetUserByID {
        id: i32,
        source: diesel::result::Error,
    },

    #[snafu(display("Insert user (name: {}, id: {}): {}", name, twitch_id, source))]
    InsertUser {
        name: String,
        twitch_id: i64,
        source: diesel::result::Error,
    },

    #[snafu(display("Updating user (twitch_id: {}): {}", twitch_id, source))]
    UpdateUser {
        twitch_id: i64,
        source: diesel::result::Error,
    },

    #[snafu(display("Insert empty user with name (name: {}): {}", name, source))]
    InsertUserWithName {
        name: String,
        source: diesel::result::Error,
    },

    GetActiveVoicemails {
        id: i32,
        source: diesel::result::Error,
    },

    SetVoicemailToInactive {
        id: i32,
        source: diesel::result::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

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
    pub fn new(
        conn: &Connection,
        twitch_id: i64,
        name: &str,
        display_name: &str,
        now: &DateTime<Local>,
    ) -> Result<Self> {
        trace!(
            "Creating new user (twitch_id: {}, name: {})",
            twitch_id,
            name
        );

        let now = now.naive_utc();

        // check if a user with the name already exists.
        // if a user is found fix the row, else add a new user.
        let user = match Self::by_name(conn, name)? {
            Some(user) => diesel::update(&user)
                .set(&FixUserWithOnlyName {
                    twitch_id,
                    display_name,
                    first_seen: &now,
                    last_seen: &now,
                })
                .get_result(conn)
                .context(UpdateUser { twitch_id })?,
            None => diesel::insert_into(users::table)
                .values(&NewUser {
                    twitch_id,
                    name,
                    display_name,
                    first_seen: &now,
                    last_seen: &now,
                })
                .get_result(conn)
                .context(InsertUser { name, twitch_id })?,
        };

        Ok(user)
    }

    pub fn with_name(conn: &Connection, name: &str) -> Result<Self> {
        trace!("Creating new empty user with name (name: {})", name);

        diesel::insert_into(users::table)
            .values(&NewUserWithName { name })
            .get_result(conn)
            .context(InsertUserWithName { name })
    }

    // TODO: check if the logic can be offloaded to the database
    pub fn bump<'a>(
        conn: &Connection,
        twitch_id: i64,
        name: &'a str,
        display_name: &'a str,
        now: &DateTime<Local>,
    ) -> Result<Self> {
        debug!("Bumping user (twitch_id: {})", twitch_id);

        // get the user from the database
        let user = match Self::by_twitch_id(conn, twitch_id)? {
            Some(user) => diesel::update(&user)
                .set(&BumpUser {
                    name,
                    display_name,
                    last_seen: &now.naive_utc(),
                })
                .get_result(conn)
                .context(UpdateUser { twitch_id })?,
            None => Self::new(&conn, twitch_id, name, display_name, now)?,
        };

        Ok(user)
    }

    pub fn by_twitch_id(conn: &Connection, twitch_id: i64) -> Result<Option<Self>> {
        trace!("Getting user (twitch_id: {})", twitch_id);

        users::table
            .filter(users::twitch_id.eq(twitch_id))
            .get_result(conn)
            .optional()
            .context(GetUserByTwitchID { twitch_id })
    }

    pub fn by_name(conn: &Connection, name: &str) -> Result<Option<Self>> {
        trace!("Getting user (name: {})", name);

        users::table
            .filter(users::name.eq(name))
            .get_result(conn)
            .optional()
            .context(GetUserByName { name })
    }

    pub fn by_id(conn: &Connection, id: i32) -> Result<Option<User>> {
        trace!("Getting user (id: {})", id);

        users::table
            .filter(users::id.eq(id))
            .get_result(conn)
            .optional()
            .context(GetUserByID { id })
    }

    pub fn pop(&self, conn: &Connection) -> Result<Vec<Voicemail>> {
        trace!("Popping voicemails (id: {})", self.id);

        let vms = Voicemail::belonging_to(self)
            .filter(
                voicemails::active
                    .eq(true)
                    .and(voicemails::scheduled.is_null()),
            )
            .get_results(conn)
            .context(GetActiveVoicemails { id: self.id })?;

        for voicemail in &vms {
            diesel::update(voicemail)
                .set(voicemails::active.eq(false))
                .execute(conn)
                .context(SetVoicemailToInactive { id: self.id })?;
        }

        Ok(vms)
    }

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
            .unwrap() // unwrap should never fail
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
