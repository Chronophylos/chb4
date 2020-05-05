use super::action::Action;
use crate::{
    context::BotContext,
    database::User,
    handler::{Handler, Twitch},
    message::{Message, MessageConsumer, MessageResult},
};
use async_trait::async_trait;
use std::sync::Arc;
use twitchchat::messages::Privmsg;

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

#[async_trait]
impl Twitch for ActionHandler {
    async fn handle(&self, msg: Arc<Privmsg<'_>>, user: &User) {
        let message = msg.data.trim().replace("\u{e0000}", ""); // remove chatterino chars

        let actions = self.actions.iter().filter(|&act| act.is_match(&message));
        let mut writer = self.context.twitchbot().read().unwrap().clone().writer();

        for action in actions {
            debug!("Found matching action {:?}", action);

            if !action.whitelisted() {
                // or the action is enabled in this channel
                trace!("Executing action");
                match action.consume(Vec::new(), Message::TwitchPrivmsg(msg.clone()), user) {
                    Ok(r) => match r {
                        MessageResult::None => {}
                        MessageResult::Message(m) => writer
                            .privmsg(&msg.channel, &m)
                            .await
                            .expect("Could not write to channel"),
                        MessageResult::MessageWithValues(m, _v) => writer
                            .privmsg(&msg.channel, &m)
                            .await
                            .expect("Could not write to channel"),
                        MessageResult::Reply(m) => {
                            writer
                                .privmsg(
                                    &msg.channel,
                                    format!("{}, {}", user.display_name_or_name(), m).as_str(),
                                )
                                .await
                                .expect("Could not write to channel");
                        }
                        MessageResult::ReplyWithValues(m, _v) => {
                            writer
                                .privmsg(
                                    &msg.channel,
                                    format!("{}, {}", user.display_name_or_name(), m).as_str(),
                                )
                                .await
                                .expect("Could not write to channel");
                        }
                    },
                    Err(e) => error!("Could not execute action ({}): {:?}", action.name(), e),
                }
            }
        }
    }
}
