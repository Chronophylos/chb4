use crate::context::Context;
use crate::database;
use snafu::{OptionExt, ResultExt, Snafu};
use std::sync::Arc;
use twitchchat::messages::Privmsg;

#[derive(Debug, Snafu)]
pub enum Error {
    GetUserFromDatabase { source: database::user::Error },
    GetIDFromMessage,
    GetDBConn { source: r2d2::Error },
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(PartialEq)]
pub enum Permission {
    Owner,
    Friend,
    Moderator,
    Broadcaster,
    User,
    Unknown,
}

impl From<u8> for Permission {
    fn from(v: u8) -> Self {
        match v {
            x if x == 255 => Self::Owner,
            x if x > 100 => Self::Friend,
            _ => Self::Unknown,
        }
    }
}

impl Permission {
    pub fn from(context: Arc<Context>, msg: Arc<Privmsg<'_>>) -> Result<Self> {
        let user_id = msg.user_id().context(GetIDFromMessage)?;
        let conn = context.pool().get().context(GetDBConn)?;
        let user = database::user::by_twitch_id(&conn, user_id).context(GetUserFromDatabase)?;
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
