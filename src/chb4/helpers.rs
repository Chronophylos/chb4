use crate::{
    database::{user, User},
    message::Message,
};
use snafu::Snafu;
use std::time::Duration;

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
    pub fn from_user(msg: Message, user: &User) -> Result<Self> {
        let permission: Permission = user.permission.into();

        if permission != Permission::Unknown {
            return Ok(permission);
        }

        match msg {
            Message::TwitchPrivmsg(msg) => {
                if msg.channel == msg.name {
                    return Ok(Permission::Broadcaster);
                }

                if msg.is_moderator() {
                    return Ok(Permission::Moderator);
                }
            }
        }

        Ok(Self::User)
    }
}

pub fn truncate_duration(dur: Duration) -> Duration {
    Duration::from_secs(dur.as_secs())
}

pub fn prettify_bool(b: bool) -> &'static str {
    if b {
        "✔"
    } else {
        "✘"
    }
}
