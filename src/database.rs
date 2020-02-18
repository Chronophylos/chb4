//! This module currently holds all database interacting functions.
//! This will change later as I move everything into their own module.

use crate::models::{BumpUser, NewUser};
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::MysqlConnection;

pub fn create_user<'a>(
    conn: &MysqlConnection,
    twitch_id: &'a str,
    name: &'a str,
    display_name: &'a str,
    now: &'a NaiveDateTime,
) {
    use crate::schema::users;
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
    twitch_id: &'a str,
    name: &'a str,
    display_name: &'a str,
    now: &'a NaiveDateTime,
) {
    use crate::schema::users;

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
