use super::*;
use crate::models::{Channel, User};
use crate::schema::channels;
use diesel::prelude::*;
use diesel::MysqlConnection;
use snafu::{ensure, ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Channel {} was not found", channel_id))]
    GetChannel {
        channel_id: u32,
        source: diesel::result::Error,
    },

    #[snafu(display("User {} has no channel entry", user))]
    UserHasNoChannel {
        user: String,
    },

    UserError {
        source: user::Error,
    },
}

impl From<user::Error> for Error {
    fn from(error: user::Error) -> Self {
        Self::UserError { source: error }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

// todo return result instead
pub fn by_name<'a>(conn: &MysqlConnection, name: &'a str) -> Result<Channel> {
    let user = user::by_name(conn, name)?;

    ensure!(
        user.channel_id.is_some(),
        UserHasNoChannel { user: user.name }
    );

    let channel_id = user.channel_id.unwrap();

    let channel = channels::table
        .filter(channels::id.eq(channel_id))
        .get_result(conn)
        .context(GetChannel { channel_id })?;

    Ok(channel)
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
pub fn join<'a>(conn: &MysqlConnection, name: &'a str) -> Result<()> {
    let user = user::by_name(conn, name)?;

    if user.channel_id.is_none() {
        todo!("create channel")
    } else {
        let channel_id = user.channel_id.unwrap();

        diesel::update(channels::table)
            .filter(channels::id.eq(channel_id))
            .set(channels::enabled.eq(true))
            .execute(conn)
            .expect("Error enabling channel");
    }

    Ok(())
}

pub fn leave<'a>(conn: &MysqlConnection, name: &'a str) -> Result<()> {
    let channel = by_name(conn, name)?;

    diesel::update(&channel)
        .set(channels::enabled.eq(false))
        .execute(conn)
        .expect("Error disabling channel");

    Ok(())
}

pub fn all_enabled(conn: &MysqlConnection) -> Vec<String> {
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
