use super::action::{Action, ActionResult};
use chb4::context::Context;
use std::sync::Arc;

pub struct ActionHandler {
    actions: Vec<Action>,
    #[allow(dead_code)]
    context: Arc<Context>,
}

impl ActionHandler {
    /// Create a new ActionHandler
    pub fn new(context: Arc<Context>) -> Self {
        Self {
            context,
            actions: Vec::new(),
        }
    }

    fn add(&mut self, action: Action) {
        self.actions.push(action);
    }

    fn add_all(&mut self, actions: Vec<Action>) {
        for action in actions {
            self.add(action);
        }
    }

    /// Handle a privmsg
    pub async fn handle_privmsg(
        &self,
        msg: Arc<twitchchat::messages::Privmsg<'_>>,
        writer: &mut twitchchat::client::Writer,
    ) {
        let message = msg.data.trim().replace("\u{e0000}", ""); // remove chatterino chars

        let actions = self.actions.iter().filter(|&act| act.is_match(&message));

        for action in actions {
            debug!("Found matching action {:?}", action);

            if !action.whitelisted() {
                // or the action is enabled in this channel
                trace!("Executing action");
                match action.execute(msg.clone()) {
                    ActionResult::Message(m) => writer
                        .privmsg(&msg.channel, &m)
                        .await
                        .expect("Could not write to channel"),
                    ActionResult::NoMessage => {}
                    ActionResult::Error(e) => {
                        error!("Could not execute action ({}): {}", action.name(), e)
                    }
                }
            }
        }
    }
}

pub fn new(context: Arc<Context>) -> ActionHandler {
    use super::aktionen;
    let mut ah = ActionHandler::new(context.clone());

    ah.add_all(aktionen::all(context.clone()));

    ah
}
