use super::*;
use crate::models::{Channel, NewChannel, NewUserWithName, User};
use crate::schema::*;
use diesel::prelude::*;
use diesel::PgConnection;
use snafu::{ensure, ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Channel {} was not found", channel_id))]
    GetChannel {
        channel_id: i32,
        source: diesel::result::Error,
    },

    #[snafu(display("User {} has no channel entry", user))]
    UserHasNoChannel {
        user: String,
    },

    UserError {
        source: user::Error,
    },

    UserNotFound,
    CreateChannel {
        source: diesel::result::Error,
    },
    SetChannelID {
        source: diesel::result::Error,
    },
    EnableChannel {
        source: diesel::result::Error,
    },
    CreateUserWithName {
        source: diesel::result::Error,
    },
}

impl From<user::Error> for Error {
    fn from(error: user::Error) -> Self {
        Self::UserError { source: error }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

// todo return result instead
pub fn by_name<'a>(conn: &PgConnection, name: &'a str) -> Result<Channel> {
    let user = match user::by_name(conn, name)? {
        Some(u) => u,
        None => return Err(Error::UserNotFound),
    };

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
pub fn join<'a>(conn: &PgConnection, name: &'a str) -> Result<()> {
    let user = match user::by_name(conn, name)? {
        Some(u) => u,
        None => diesel::insert_into(users::table)
            .values(&NewUserWithName { name })
            .get_result(conn)
            .context(CreateUserWithName)?,
    };

    if user.channel_id.is_none() {
        let channel_id: i32 = diesel::insert_into(channels::table)
            .values(&NewChannel {
                twitch_id: user.twitch_id,
                enabled: true,
            })
            .returning(channels::id)
            .get_result(conn)
            .context(CreateChannel)?;

        diesel::update(&user)
            .set(users::channel_id.eq(channel_id))
            .execute(conn)
            .context(SetChannelID)?;
    } else {
        let channel_id = user.channel_id.unwrap();

        diesel::update(channels::table)
            .filter(channels::id.eq(channel_id))
            .set(channels::enabled.eq(true))
            .execute(conn)
            .context(EnableChannel)?;
    }

    Ok(())
}

pub fn leave<'a>(conn: &PgConnection, name: &'a str) -> Result<()> {
    let channel = by_name(conn, name)?;

    diesel::update(&channel)
        .set(channels::enabled.eq(false))
        .execute(conn)
        .expect("Error disabling channel");

    Ok(())
}

pub fn all_enabled(conn: &PgConnection) -> Vec<String> {
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
