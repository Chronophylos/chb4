use super::action::Action;
use crate::{
    context::BotContext,
    database::User,
    handler::{Handler, SimpleHandler, Twitch},
    message::{Message, MessageConsumer, MessageResult},
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::sync::Arc;
use thiserror::Error;
use twitchchat::messages::Privmsg;

#[derive(Debug, Error)]
pub enum ActionHandlerError {
    #[error("Could not execute action (name: {0})")]
    ExecuteAction(String),

    #[error("Could not send privmsg")]
    SendPrivmsg,
}

pub struct ActionHandler {
    actions: Vec<Arc<Action>>,

    context: Arc<BotContext>,
}

impl ActionHandler {
    /// Create a new ActionHandler
    pub fn new(context: Arc<BotContext>, actions: Vec<Arc<Action>>) -> Self {
        Self { context, actions }
    }
}

impl Handler<Action> for ActionHandler {
    fn get(&self, name: String) -> Option<Arc<Action>> {
        self.actions.iter().find(|&a| a.name() == name).cloned()
    }
}

impl SimpleHandler for ActionHandler {
    fn name(&self) -> &str {
        "action"
    }
}

#[async_trait]
impl Twitch for ActionHandler {
    async fn handle(&self, msg: Arc<Privmsg<'_>>, user: &User) -> Result<()> {
        let message = msg.data.trim().replace("\u{e0000}", ""); // remove chatterino chars

        let actions = self.actions.iter().filter(|&act| act.is_match(&message));
        let mut writer = self.context.twitchbot().writer();

        for action in actions {
            // do not write to log on every message
            debug!("Found matching action {:?}", action);

            if action.whitelisted() {
                debug!(
                    "Action is not whitelisted in this channel (name: {}, channel: {})",
                    action.name(),
                    msg.channel
                );
                return Ok(());
            }

            trace!("Executing action");

            match action
                .consume(
                    self.context.clone(),
                    Vec::new(),
                    Message::TwitchPrivmsg(msg.clone()),
                    user,
                )
                .context(ActionHandlerError::ExecuteAction(action.name().to_owned()))?
            {
                MessageResult::None => Ok(()),
                MessageResult::Reply(m) => writer
                    .privmsg(
                        &msg.channel,
                        format!("{}, {}", user.display_name_or_name(), m).as_str(),
                    )
                    .await
                    .context(ActionHandlerError::SendPrivmsg),
                MessageResult::Message(m) => writer
                    .privmsg(&msg.channel, &m)
                    .await
                    .context(ActionHandlerError::SendPrivmsg),
                MessageResult::Error(m) => writer
                    .privmsg(&msg.channel, format!("Error: {}", m))
                    .await
                    .context(ActionHandlerError::SendPrivmsg),
                MessageResult::MissingArgument(a) => writer
                    .privmsg(&msg.channel, format!("Missing argument `{}`", a))
                    .await
                    .context(ActionHandlerError::SendPrivmsg),
            }?;
        }

        Ok(())
    }
}
