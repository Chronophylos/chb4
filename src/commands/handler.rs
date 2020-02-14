//! Everything needed to handle and create commands
use super::command::Command;
use std::collections::HashMap;

pub struct CommandHandler {
    commands: HashMap<String, Command>,
    // translate aliases to command names
    aliases: HashMap<String, String>,
    /// The prefix to use when checking for commands in a message.
    prefix: char,
}

impl Default for CommandHandler {
    fn default() -> Self {
        Self {
            commands: HashMap::new(),
            aliases: HashMap::new(),
            prefix: '§',
        }
    }
}

impl CommandHandler {
    /// Create a new CommandHandler
    pub fn new() -> Self {
        Self::default()
    }

    /// Add `command` to the CommandHandler by saving it in `commands` with `name` as key.
    /// Save all of it's aliases in `aliases` with `name` as value and the respective alias as key.
    fn add_command(&mut self, command: Command) {
        let name = command.name.clone();

        // insert aliases into alias map
        for alias in &command.aliases {
            self.aliases.insert(alias.to_owned(), name.clone());
        }

        // insert command into command map
        self.commands.insert(name, command);
    }

    /// Get a command by `name`. This can either be the command name or any of it's aliases.
    fn get_command(&self, name: String) -> Option<&Command> {
        let name = match self.aliases.get(&name) {
            Some(n) => n,
            _ => &name,
        };
        self.commands.get(name)
    }

    /// Handle a privmsg
    pub async fn handle_privmsg(
        &self,
        msg: &std::sync::Arc<twitchchat::messages::Privmsg<'_>>,
        writer: &mut twitchchat::client::Writer,
    ) {
        let message = msg.data.trim().replace("\u{e0000}", ""); // remove chatterino chars
        let words: Vec<&str> = message.split_whitespace().collect();
        let mut command_name = words[0].to_owned();
        let prefix = command_name.remove(0);

        if prefix != self.prefix {
            trace!("Dropping message because prefix was not found");
            return;
        }

        let args = &words[1..];
        debug!("Command: {} Args: {:?}", command_name, args);

        match self.get_command(command_name) {
            Some(cmd) => {
                debug!("Found matching command {}", cmd.name);
                if !cmd.whitelisted {
                    // or the command is enabled in this channel
                    debug!("Executing command");
                    match cmd.execute(args.to_vec()) {
                        Ok(r) => {
                            if r.message.is_some() {
                                writer
                                    .privmsg(&msg.channel, &r.message.unwrap())
                                    .await
                                    .expect("Could not write to channel");
                            }
                        }
                        Err(e) => error!("Could not execute command (name: {}): {}", cmd.name, e),
                    }
                }
            }
            None => debug!("No matching command found"),
        }
    }
}

pub fn new() -> CommandHandler {
    use super::test;
    let mut ch = CommandHandler::new();

    ch.add_command(test::command());

    ch
}
