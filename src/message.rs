use crate::{context::BotContext, database::User};
use anyhow::Result;
use std::sync::Arc;
use twitchchat::messages::Privmsg;

pub enum MessageResult {
    None,
    Reply(String),
    Message(String),
    Error(String),
    MissingArgument(&'static str),
}

pub trait MessageConsumer: Send + Sync {
    fn name(&self) -> &str;
    fn whitelisted(&self) -> bool;

    fn consume(
        &self,
        context: Arc<BotContext>,
        args: Vec<String>,
        msg: Message,
        user: &User,
    ) -> Result<MessageResult>;
}

pub enum Message<'a> {
    TwitchPrivmsg(Arc<Privmsg<'a>>),
}

impl Message<'_> {
    pub fn channel(&self) -> &str {
        match self {
            Self::TwitchPrivmsg(msg) => &msg.channel,
        }
    }

    pub fn twitch_id(&self) -> Option<u64> {
        match self {
            Self::TwitchPrivmsg(msg) => msg.user_id(),
        }
    }

    pub fn sent_ts(&self) -> u64 {
        match self {
            Self::TwitchPrivmsg(msg) => msg.tmi_sent_ts().unwrap_or(0),
        }
    }

    pub fn color(&self) -> String {
        match self {
            Self::TwitchPrivmsg(msg) => format!("{}", msg.color().unwrap()),
        }
    }
}
