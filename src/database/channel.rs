use crate::models::Channel;
use crate::schema::channels;
use diesel::prelude::*;
use diesel::MysqlConnection;

pub fn get_channel<'a>(conn: &MysqlConnection, name: &'a str) {
    use user::get_user_by_name;

    let user = get_user_by_name(name);

    Channel::belonging_to(&user)
}

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
