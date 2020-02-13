/// stub
struct Action {
    name: String,
    #[allow(dead_code)]
    aliases: Vec<String>,
    whitelisted: bool,
}

pub struct ActionHandler {
    actions: Vec<Action>,
    prefix: char,
}

impl Default for ActionHandler {
    fn default() -> Self {
        Self {
            actions: vec![Action {
                name: String::from("dummy"),
                aliases: vec![],
                whitelisted: true,
            }],
            prefix: '§',
        }
    }
}

impl ActionHandler {
    /// Create a new ActionHandler
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_privmsg(
        &self,
        msg: std::sync::Arc<twitchchat::messages::Privmsg<'_>>,
        _writer: &twitchchat::client::Writer,
    ) {
        let words: Vec<&str> = msg.data.trim().split_whitespace().collect();
        let mut command = words[0].to_owned();
        let prefix = command.remove(0);

        if prefix != self.prefix {
            trace!("Dropping message because prefix was not found");
            return;
        }

        let args = &words[1..];
        debug!("Command: {} Args: {:?}", command, args);

        let action = self.actions.iter().find(|&a| command == a.name);
        match action {
            Some(a) => {
                debug!("Found matching action {}", a.name);
                if a.whitelisted {
                    debug!("Executing action");
                }
            }
            None => debug!("No matching action found"),
        }
    }
}

pub fn new() -> ActionHandler {
    ActionHandler::new()
}
