use crate::models::{BumpUser, NewUser, User};
use crate::schema::users;
use chrono::prelude::*;
use diesel::prelude::*;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    GetUserByTwitchID {
        twitch_id: i64,
        source: diesel::result::Error,
    },

    GetUserByName {
        name: String,
        source: diesel::result::Error,
    },

    InsertUser {
        name: String,
        twitch_id: i64,
        source: diesel::result::Error,
    },

    UpdateUser {
        twitch_id: i64,
        source: diesel::result::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn create<'a>(
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

    let now = now.naive_utc();

    let user = diesel::insert_into(users::table)
        .values(&NewUser {
            twitch_id,
            name,
            display_name,
            first_seen: &now,
            last_seen: &now,
        })
        .get_result(conn)
        .context(InsertUser { name, twitch_id })?;

    Ok(user)
}

pub fn bump<'a>(
    conn: &PgConnection,
    twitch_id: i64,
    name: &'a str,
    display_name: &'a str,
    now: &DateTime<Local>,
) -> Result<User> {
    debug!("Bumping user (twitch_id: {})", twitch_id);

    let user = self::by_twitch_id(conn, twitch_id)?;

    if user.is_some() {
        trace!("User found -> bumping user");

        let user = diesel::update(users::table)
            .filter(users::twitch_id.eq(twitch_id))
            .set(&BumpUser {
                name,
                display_name,
                last_seen: &now.naive_utc(),
            })
            .get_result(conn)
            .context(UpdateUser { twitch_id })?;

        return Ok(user);
    } else {
        trace!("User not found -> creating new user");

        let user = create(&conn, twitch_id, name, display_name, now)?;

        return Ok(user);
    }
}

pub fn by_twitch_id<'a>(conn: &PgConnection, twitch_id: i64) -> Result<Option<User>> {
    trace!("Getting user (twitch_id: {})", twitch_id);

    users::table
        .filter(users::twitch_id.eq(twitch_id))
        .get_result::<User>(conn)
        .optional()
        .context(GetUserByTwitchID { twitch_id })
}

pub fn by_name<'a>(conn: &PgConnection, name: &'a str) -> Result<Option<User>> {
    trace!("Getting user (name: {})", name);

    users::table
        .filter(users::name.eq(name))
        .get_result(conn)
        .optional()
        .context(GetUserByName { name })
}
