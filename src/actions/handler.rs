use super::action::{Action, ActionResult};

pub struct ActionHandler<'a> {
    actions: Vec<Action<'a>>,
}

impl<'a> ActionHandler<'a> {
    /// Create a new ActionHandler
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    fn add_action(&mut self, action: Action<'a>) {
        self.actions.push(action);
    }

    /// Handle a privmsg
    pub async fn handle_privmsg(
        &self,
        msg: &std::sync::Arc<twitchchat::messages::Privmsg<'_>>,
        writer: &mut twitchchat::client::Writer,
    ) {
        let message = msg.data.trim().replace("\u{e0000}", ""); // remove chatterino chars

        let actions = self
            .actions
            .iter()
            .filter(|&act| act.regex().is_match(&message));
        for action in actions {
            debug!("Found matching action {}", action.name());

            if !action.whitelisted() {
                // or the command is enabled in this channel
                trace!("Executing action");
                match action.execute(msg) {
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

pub fn new<'a>() -> ActionHandler<'a> {
    use super::aktionen;
    let mut ah = ActionHandler::new();

    ah.add_action(aktionen::test());

    ah
}
