use crate::models::{Channel, User};
use crate::schema::{channels, users};
use diesel::prelude::*;
use diesel::MysqlConnection;

// todo return result instead
pub fn get_channel<'a>(conn: &MysqlConnection, name: &'a str) -> Option<Channel> {
    use super::get_user_by_name;

    let user = get_user_by_name(conn, name);

    if user.channel_id.is_none() {
        return None;
    }

    let channel = channels::table
        .filter(channels::id.eq(user.channel_id.unwrap()))
        .get_result(conn)
        .expect("Could not get channel belonging to user");

    Some(channel)
}

/// Join a channel with `name`. If the channel already exists simply update `enabled` to `true`
/// regardless of it's current value. If the channel does not exists check if a user with `name`
/// exists and create a new channel pointing to that user and set enabled to `true`. If no user
/// exists create a new user with only the name and continue with creating the channel.
///
/// See below for a illustration:
///
/// ```markdeep
///                                                               .-----------------------.
///                                                               | Set enabled to `true` |
///                                                               '-----------------------'
///                                                                          ^
///                                                                          |
///                                .                                         .
///                               / \                                       / \
///                              /   \                                     /   \
/// .----------------------.    /     \  Y  .------------------------.    /     \
/// | Get user with `name` +-->+Exists?+--->| Get associated channel +-->+Exists?+
/// '----------------------'    \     /     '------------------------'    \     /
///                              \   /                   ^                 \   /
///                               \ /                    |                  \ /
///                                .                     |                   .
///                                |N                    |                   |N
///                                v                     |                   v
///                     .-------------------------.      |      .---------------------------.
///                     | Create user with `name` +------+      | Create channel of user    |
///                     '-------------------------'             | and set enabled to `true` |
///                                                             '---------------------------'
/// ```
// todo: return result
pub fn join_channel<'a>(conn: &MysqlConnection, name: &'a str) {
    let channel = match get_channel(conn, name) {
        Some(c) => {
            diesel::update(&c)
                .set(channels::enabled.eq(true))
                .execute(conn)
                .expect("Error enabling channel");
        }
        None => todo!("create channel with enabled = true"),
    };
}

pub fn leave_channel<'a>(conn: &MysqlConnection, name: &'a str) {
    let channel = get_channel(conn, name).unwrap();

    diesel::update(&channel)
        .set(channels::enabled.eq(false))
        .execute(conn)
        .expect("Error disabling channel");
}

pub fn get_enabled_channels(conn: &MysqlConnection) -> Vec<String> {
    let channels = channels::table
        .filter(channels::enabled.eq(true))
        .load::<Channel>(conn)
        .unwrap();

    let users = User::belonging_to(&channels)
        .load::<User>(conn)
        .unwrap()
        .grouped_by(&channels);

    channels
        .into_iter()
        .zip(users)
        .map(|(_, u)| u.get(0).unwrap().name.clone())
        .collect()
}
