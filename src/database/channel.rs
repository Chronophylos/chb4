use crate::models::Channel;
use crate::schema::channels;
use diesel::prelude::*;
use diesel::MysqlConnection;

pub fn get_channel<'a>(conn: &MysqlConnection, name: &'a str) {
    use super::get_user_by_name;

    let user = get_user_by_name(conn, name);

    Channel::belonging_to(&user)
        .get_result(conn)
        .expect("Could not get channel belonging to user")
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
pub fn join_channel<'a>(conn: &MysqlConnection, name: &'a str) {
    let channel = get_channel(conn, name);

    diesel::update(channel)
        .set(channels::enabled.eq(true))
        .execute(conn)
        .expect("Error enabling channel");
}

pub fn leave_channel<'a>(conn: &MysqlConnection, name: &'a str) {
    let channel = get_channel(conn, name);

    diesel::update(channel)
        .set(channels::enabled.eq(false))
        .execute(conn)
        .expect("Error disabling channel");
}

pub fn get_enabled_channels(conn: &MysqlConnection) -> Vec<Channel> {
    channels::table
        .filter(channels::enabled.eq(true))
        .get_result(conn)
        .expect("Error getting channels")
}
