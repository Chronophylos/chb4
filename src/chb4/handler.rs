use crate::{database::User, message::MessageConsumer};
use async_trait::async_trait;
use std::sync::Arc;
use twitchchat::messages::Privmsg;

pub trait Handler: Twitch + Send + Sync {
    fn get(&self, name: String) -> Option<&dyn MessageConsumer>;
}

#[async_trait]
pub trait Twitch {
    async fn handle(&self, msg: Arc<Privmsg<'_>>, user: &User);
}
