use crate::models::{BumpUser, NewUser, User};
use crate::schema::users;
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::MysqlConnection;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    GetUserByTwitchID {
        twitch_id: u64,
        source: diesel::result::Error,
    },

    GetUserByName {
        name: String,
        source: diesel::result::Error,
    },

    InsertUser {
        name: String,
        twitch_id: u64,
        source: diesel::result::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn create<'a>(
    conn: &MysqlConnection,
    twitch_id: u64,
    name: &str,
    display_name: &str,
    now: &DateTime<Local>,
) -> Result<bool> {
    trace!(
        "Creating new user (twitch_id: {}, name: {})",
        twitch_id,
        name
    );

    let now = now.naive_utc();

    let inserted = diesel::insert_into(users::table)
        .values(&NewUser {
            twitch_id,
            name,
            display_name,
            first_seen: &now,
            last_seen: &now,
        })
        .execute(conn)
        .context(InsertUser { name, twitch_id })?;

    Ok(inserted == 1)
}

pub fn bump<'a>(
    conn: &MysqlConnection,
    twitch_id: u64,
    name: &'a str,
    display_name: &'a str,
    now: &DateTime<Local>,
) -> Result<User> {
    debug!("Bumping user (twitch_id: {})", twitch_id);

    let user_exits: i64 = users::table
        .filter(users::twitch_id.eq(twitch_id))
        .count()
        .get_result(conn)
        .context(GetUserByTwitchID { twitch_id })?;

    if user_exits == 1 {
        trace!("User found -> bumping user");
        diesel::update(users::table)
            .filter(users::twitch_id.eq(twitch_id))
            .set(&BumpUser {
                name,
                display_name,
                last_seen: &now.naive_utc(),
            })
            .execute(conn)
            .expect("Error bumping user");
    } else {
        trace!("User not found -> creating new user");
        create(&conn, twitch_id, name, display_name, now)?;
    }

    Ok(by_twitch_id(conn, twitch_id)?)
}

pub fn by_twitch_id<'a>(conn: &MysqlConnection, twitch_id: u64) -> Result<User> {
    trace!("Getting user (twitch_id: {})", twitch_id);

    users::table
        .filter(users::twitch_id.eq(twitch_id))
        .limit(1)
        .get_result(conn)
        .context(GetUserByTwitchID { twitch_id })
}

pub fn by_name<'a>(conn: &MysqlConnection, name: &'a str) -> Result<User> {
    trace!("Getting user (name: {})", name);

    users::table
        .filter(users::name.eq(name))
        .limit(1)
        .get_result(conn)
        .context(GetUserByName { name })
}
