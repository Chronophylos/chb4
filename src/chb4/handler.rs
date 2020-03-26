use crate::{database::User, message::MessageConsumer};
use async_trait::async_trait;
use std::sync::Arc;
use twitchchat::messages::Privmsg;

pub trait Handler<T>: Twitch + Send + Sync
where
    T: MessageConsumer,
{
    fn get(&self, name: String) -> Option<Arc<T>>;
}

#[async_trait]
pub trait Twitch: Send + Sync {
    async fn handle(&self, msg: Arc<Privmsg<'_>>, user: &User);
}
