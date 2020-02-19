use crate::models::{BumpUser, NewUser, User};
use crate::schema::users;
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::MysqlConnection;

pub fn create_user<'a>(
    conn: &MysqlConnection,
    twitch_id: u64,
    name: &'a str,
    display_name: &'a str,
    now: &'a NaiveDateTime,
) {
    trace!(
        "Creating new user (twitch_id: {}, name: {})",
        twitch_id,
        name
    );

    diesel::insert_into(users::table)
        .values(&NewUser {
            twitch_id,
            name,
            display_name,
            first_seen: now,
            last_seen: now,
        })
        .execute(conn)
        .expect("Error saving user");
}

pub fn bump_user<'a>(
    conn: &MysqlConnection,
    twitch_id: u64,
    name: &'a str,
    display_name: &'a str,
    now: &'a NaiveDateTime,
) {
    debug!("Bumping user (twitch_id: {})", twitch_id);

    let user_exits: i64 = users::table
        .filter(users::twitch_id.eq(twitch_id))
        .count()
        .get_result(conn)
        .expect("Error getting user");

    if user_exits == 1 {
        trace!("User found -> bumping user");
        diesel::update(users::table)
            .filter(users::twitch_id.eq(twitch_id))
            .set(&BumpUser {
                name,
                display_name,
                last_seen: now,
            })
            .execute(conn)
            .expect("Error bumping user");
    } else {
        trace!("User not found -> creating new user");
        create_user(&conn, twitch_id, name, display_name, now);
    }
}

pub fn get_user<'a>(conn: &MysqlConnection, twitch_id: u64) -> User {
    trace!("Getting user (twitch_id: {})", twitch_id);

    users::table
        .filter(users::twitch_id.eq(twitch_id))
        .limit(1)
        .get_result(conn)
        .expect("Error getting user")
}

pub fn get_user_by_name<'a>(conn: &MysqlConnection, name: &'a str) -> User {
    trace!("Getting user (name: {})", name);

    users::table
        .filter(users::name.eq(name))
        .limit(1)
        .get_result(conn)
        .expect("Could not get User by name")
}
