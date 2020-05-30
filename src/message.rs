use crate::{context::BotContext, database::User};
use std::sync::Arc;
use thiserror::Error;
use twitchchat::messages::Privmsg;

pub type Result = std::result::Result<MessageResult, MessageError>;

#[derive(Debug, Error)]
#[error("{0}")]
pub struct MessageError(String);

impl From<String> for MessageError {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for MessageError {
    fn from(s: &str) -> Self {
        Self(String::from(s))
    }
}

pub enum MessageResult {
    None,
    Message(String),
    MessageWithValues(String, Vec<String>),
    Reply(String),
    ReplyWithValues(String, Vec<String>),
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
    ) -> Result;
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
            Self::TwitchPrivmsg(msg) => Some(msg.user_id().unwrap()),
        }
    }

    pub fn sent_ts(&self) -> u64 {
        match self {
            Self::TwitchPrivmsg(msg) => msg.tmi_sent_ts().unwrap(),
        }
    }

    pub fn color(&self) -> String {
        match self {
            Self::TwitchPrivmsg(msg) => format!("{}", msg.color().unwrap()),
        }
    }
}
