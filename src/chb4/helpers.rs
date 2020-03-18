use crate::context::Context;
use crate::database::{user, User};
use snafu::{OptionExt, ResultExt, Snafu};
use std::convert::TryInto;
use std::sync::Arc;
use twitchchat::messages::Privmsg;

#[derive(Debug, Snafu)]
pub enum Error {
    GetUser { source: user::Error },
    GetIDFromMessage,
    ConvertUserID { source: std::num::TryFromIntError },
    UserNotFound,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Permission {
    // Lowest
    Unknown,
    User,
    Broadcaster,
    Moderator,
    Friend,
    Owner,
    // Highest
}

impl From<i16> for Permission {
    fn from(v: i16) -> Self {
        match v {
            x if x >= 1337 => Self::Owner,
            x if x > 100 => Self::Friend,
            _ => Self::Unknown,
        }
    }
}

impl Permission {
    pub fn from(context: Arc<Context>, msg: Arc<Privmsg<'_>>) -> Result<Self> {
        let uid = msg.user_id().context(GetIDFromMessage)?;
        let id: i64 = uid.try_into().context(ConvertUserID)?;
        let conn = context.conn();
        let user = match User::by_twitch_id(&conn, id).context(GetUser)? {
            Some(u) => u,
            None => return Err(Error::UserNotFound),
        };

        Self::from_user(msg, &user)
    }

    pub fn from_user(msg: Arc<Privmsg<'_>>, user: &User) -> Result<Self> {
        let permission: Permission = user.permission.into();

        if permission != Permission::Unknown {
            return Ok(permission);
        }

        if msg.channel == msg.name {
            return Ok(Permission::Broadcaster);
        }

        if msg.is_moderator() {
            return Ok(Permission::Moderator);
        }

        Ok(Self::User)
    }
}
