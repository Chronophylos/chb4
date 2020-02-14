use regex::Regex;

struct Action {
    name: String,
    regex: Regex,
    whitelisted: bool,
}

pub struct ActionHandler {
    actions: Vec<Action>,
}

impl Default for ActionHandler {
    fn default() -> Self {
        Self {
            actions: vec![Action {
                name: String::from("test"),
                regex: Regex::new(r"^test").unwrap(),
                whitelisted: true,
            }],
        }
    }
}

impl ActionHandler {
    /// Create a new ActionHandler
    pub fn new() -> Self {
        Self::default()
    }

    /// Handle a privmsg
    pub fn handle_privmsg(
        &self,
        msg: &std::sync::Arc<twitchchat::messages::Privmsg<'_>>,
        _writer: &twitchchat::client::Writer,
    ) {
        let message = msg.data.to_string();

        debug!("Message: {}", message);

        let actions = self
            .actions
            .iter()
            .filter(|&act| act.regex.is_match(&message));
        for action in actions {
            debug!("Found matching action {}", action.name);

            if action.whitelisted {
                debug!("Executing action");
            }
        }
    }
}

pub fn new() -> ActionHandler {
    ActionHandler::new()
}
