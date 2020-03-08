use super::*;
use crate::models::{Channel, NewChannel, NewUserWithName, User};
use crate::schema::*;
use diesel::prelude::*;
use diesel::PgConnection;
use snafu::{OptionExt, ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Getting channel (channel_id: {}): {}", channel_id, source))]
    GetChannel {
        channel_id: i32,
        source: diesel::result::Error,
    },

    #[snafu(display("User has no channel entry (name: {})", name))]
    UserHasNoChannel {
        name: String,
    },

    #[snafu(display("User not found (name: {})", name))]
    UserNotFound {
        name: String,
    },

    #[snafu(display("Inserting channel: {}", source))]
    CreateChannel {
        source: diesel::result::Error,
    },

    #[snafu(display("Setting channel_id of user (id: {}): {}", id, source))]
    SetChannelID {
        id: i32,
        source: diesel::result::Error,
    },

    #[snafu(display("Enabling channel: {}", source))]
    EnableChannel {
        source: diesel::result::Error,
    },

    #[snafu(display("Disabling channel: {}", source))]
    DisableChannel {
        source: diesel::result::Error,
    },

    #[snafu(display("Creating new user (name: {}): {}", name, source))]
    CreateUserWithName {
        name: String,
        source: diesel::result::Error,
    },

    #[snafu(display("Getting enabled channels: {}", source))]
    GetEnabledChannels {
        source: diesel::result::Error,
    },

    #[snafu(display("Getting users owning channels: {}", source))]
    GetAssociatedUsers {
        source: diesel::result::Error,
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
pub fn by_name<'a>(conn: &PgConnection, name: &'a str) -> Result<Channel> {
    let user: Option<User> = user::by_name(conn, name)?;
    let user: User = user.context(UserNotFound {
        name: name.to_owned(),
    })?;

    let channel_id = user
        .channel_id
        .context(UserHasNoChannel { name: user.name })?;

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
            .context(CreateUserWithName { name })?,
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
            .context(SetChannelID { id: user.id })?;
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
        .context(DisableChannel)?;

    Ok(())
}

pub fn all_enabled(conn: &PgConnection) -> Result<Vec<String>> {
    let channels = channels::table
        .filter(channels::enabled.eq(true))
        .load::<Channel>(conn)
        .context(GetEnabledChannels)?;

    let users = User::belonging_to(&channels)
        .load::<User>(conn)
        .context(GetAssociatedUsers)?
        .grouped_by(&channels);

    Ok(channels
        .into_iter()
        .zip(users)
        .map(|(_, u)| u.get(0).unwrap().name.clone())
        .collect())
}
