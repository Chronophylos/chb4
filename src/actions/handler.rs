use super::action::{Action, ActionResult};

pub struct ActionHandler {
    actions: Vec<Action>,
}

impl ActionHandler {
    /// Create a new ActionHandler
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    fn add_action(&mut self, action: Action) {
        self.actions.push(action);
    }

    /// Handle a privmsg
    pub async fn handle_privmsg(
        &self,
        msg: &std::sync::Arc<twitchchat::messages::Privmsg<'_>>,
        writer: &mut twitchchat::client::Writer,
    ) {
        let message = msg.data.to_string();

        debug!("Message: {}", message);

        let actions = self
            .actions
            .iter()
            .filter(|&act| act.regex().is_match(&message));
        for action in actions {
            debug!("Found matching action {}", action.name());

            if action.whitelisted() {
                debug!("Executing action");
                match action.execute(msg) {
                    ActionResult::Message(m) => writer.privmsg(&msg.channel, &m).await.unwrap(),
                    ActionResult::NoMessage => {}
                    ActionResult::Error(e) => {
                        error!("Could not execute action ({}): {}", action.name(), e)
                    }
                }
            }
        }
    }
}

pub fn new() -> ActionHandler {
    use super::aktionen;
    let mut ah = ActionHandler::new();

    ah.add_action(aktionen::test());

    ah
}
