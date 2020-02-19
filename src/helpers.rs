use crate::context::Context;
use crate::database;
use std::sync::Arc;
use twitchchat::messages::Privmsg;

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
    pub fn from(context: Arc<Context>, msg: Arc<Privmsg<'_>>) -> Self {
        let user_id = msg.user_id().unwrap();
        let conn = context.pool().get().unwrap();
        let user = database::get_user(&conn, user_id);
        let permission: Permission = user.permission.into();

        if permission != Permission::Unknown {
            return permission;
        }

        if msg.channel == msg.name {
            return Permission::Broadcaster;
        }

        if msg.is_moderator() {
            return Permission::Moderator;
        }

        Self::User
    }
}
