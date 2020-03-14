use crate::models::{BumpUser, FixUserWithOnlyName, NewUser, NewUserWithName, User};
use crate::schema::users;
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

    CreateUserWithName {
        name: String,
        source: diesel::result::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn new(
    conn: &PgConnection,
    twitch_id: i64,
    name: &str,
    display_name: &str,
    now: &DateTime<Local>,
) -> Result<User> {
    trace!(
        "Creating new user (twitch_id: {}, name: {})",
        twitch_id,
        name
    );

    // check if use with the same name exists

    let now = now.naive_utc();
    let user = match self::by_name(conn, name)? {
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

pub fn with_name(conn: &PgConnection, name: &str) -> Result<User> {
    trace!("Creating new empty user with name (name: {}", name);

    diesel::insert_into(users::table)
        .values(&NewUserWithName { name })
        .get_result(conn)
        .context(CreateUserWithName { name })
}

// TODO: check if the logic can be offloaded to the database
pub fn bump<'a>(
    conn: &PgConnection,
    twitch_id: i64,
    name: &'a str,
    display_name: &'a str,
    now: &DateTime<Local>,
) -> Result<User> {
    debug!("Bumping user (twitch_id: {})", twitch_id);

    // get the user from the database
    let user = match self::by_twitch_id(conn, twitch_id)? {
        Some(user) => diesel::update(&user)
            .set(&BumpUser {
                name,
                display_name,
                last_seen: &now.naive_utc(),
            })
            .get_result(conn)
            .context(UpdateUser { twitch_id })?,
        None => new(&conn, twitch_id, name, display_name, now)?,
    };

    Ok(user)
}

pub fn by_twitch_id(conn: &PgConnection, twitch_id: i64) -> Result<Option<User>> {
    trace!("Getting user (twitch_id: {})", twitch_id);

    users::table
        .filter(users::twitch_id.eq(twitch_id))
        .get_result::<User>(conn)
        .optional()
        .context(GetUserByTwitchID { twitch_id })
}

pub fn by_name(conn: &PgConnection, name: &str) -> Result<Option<User>> {
    trace!("Getting user (name: {})", name);

    users::table
        .filter(users::name.eq(name))
        .get_result(conn)
        .optional()
        .context(GetUserByName { name })
}
