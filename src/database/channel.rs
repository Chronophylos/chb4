use super::{user, Connection, User};
use crate::schema::*;
use diesel::prelude::*;
use snafu::{OptionExt, ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Getting channel (id: {}): {}", id, source))]
    GetChannelByID {
        id: i32,
        source: diesel::result::Error,
    },

    #[snafu(display("Getting channel (name: {}): {}", name, source))]
    GetChannelByName {
        name: String,
        source: diesel::result::Error,
    },

    #[snafu(display("User has no channel entry (user_id: {})", user_id))]
    UserHasNoChannel {
        user_id: i32,
    },

    #[snafu(display("User not found (name: {})", name))]
    UserNotFound {
        name: String,
    },

    #[snafu(display("Getting user (channel_id: {}): {}", channel_id, source))]
    GetUserByChannelID {
        channel_id: i32,
        source: diesel::result::Error,
    },

    GetUserBelongingToChannel {
        source: diesel::result::Error,
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
        source: super::user::Error,
    },
}

impl From<user::Error> for Error {
    fn from(error: user::Error) -> Self {
        Self::UserError { source: error }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Queryable, Identifiable)]
#[table_name = "channels"]
pub struct Channel {
    pub id: i32,
    pub twitch_id: Option<i64>,
    pub enabled: bool,
    pub paused: bool,
}

impl Channel {
    /// Get a `Channel` by the owners name.
    pub fn by_name<'a>(conn: &Connection, name: &'a str) -> Result<Option<Channel>> {
        trace!("Getting channel (name: {})", name);

        let user = User::by_name(conn, name)?.context(UserNotFound { name })?;

        Self::by_id(
            conn,
            user.channel_id
                .context(UserHasNoChannel { user_id: user.id })?,
        )
    }

    pub fn by_id(conn: &Connection, id: i32) -> Result<Option<Self>> {
        trace!("Getting channel (id: {})", id);

        channels::table
            .filter(channels::id.eq(id))
            .get_result(conn)
            .optional()
            .context(GetChannelByID { id })
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
    pub fn join<'a>(conn: &Connection, name: &'a str) -> Result<()> {
        let user = match User::by_name(conn, name)? {
            Some(u) => u,
            None => User::with_name(conn, name)?,
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

    pub fn all_enabled(conn: &Connection) -> Result<Vec<String>> {
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

    pub fn leave(&self, conn: &Connection) -> Result<()> {
        diesel::update(self)
            .set(channels::enabled.eq(false))
            .execute(conn)
            .context(DisableChannel)?;

        Ok(())
    }

    pub fn name(&self, conn: &Connection) -> Result<String> {
        let user: User = User::belonging_to(self)
            .get_result(conn)
            .context(GetUserBelongingToChannel)?;

        Ok(user.name)
    }
}

#[derive(Insertable)]
#[table_name = "channels"]
pub struct NewChannel {
    pub twitch_id: Option<i64>,
    pub enabled: bool,
}

#[derive(Queryable, Identifiable, Associations)]
#[belongs_to(Channel)]
#[table_name = "channel_action_filters"]
pub struct ChannelActionFilter {
    pub id: i32,
    pub channel_id: i32,
    pub action_name: String,
    pub enable_action: bool,
}

#[derive(Queryable, Identifiable, Associations)]
#[belongs_to(Channel)]
#[table_name = "channel_command_filters"]
pub struct ChannelCommandFilter {
    pub id: i32,
    pub channel_id: i32,
    pub command_name: String,
    pub enable_command: bool,
}
