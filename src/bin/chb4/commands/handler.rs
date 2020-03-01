//! Everything needed to handle and create commands
use super::command::{Command, CommandResult};
use chb4::context::Context;
use std::collections::HashMap;
use std::sync::Arc;

pub struct CommandHandler {
    commands: HashMap<String, Command>,
    // translate aliases to command names
    aliases: HashMap<String, String>,

    /// The prefix to use when checking for commands in a message.
    prefix: char,

    #[allow(dead_code)]
    context: Arc<Context>,
}

impl CommandHandler {
    /// Create a new CommandHandler
    pub fn new(context: Arc<Context>) -> Self {
        Self {
            context,
            commands: HashMap::new(),
            aliases: HashMap::new(),
            prefix: '§',
        }
    }

    /// Add `command` to the CommandHandler by saving it in `commands` with `name` as key.
    /// Save all of it's aliases in `aliases` with `name` as value and the respective alias as key.
    fn add(&mut self, command: Command) {
        // insert aliases into alias map
        for alias in command.aliases() {
            self.aliases.insert(alias.to_owned(), command.name());
        }

        // insert command into command map
        self.commands.insert(command.name(), command);
    }

    fn add_all(&mut self, commands: Vec<Command>) {
        for command in commands {
            self.add(command);
        }
    }

    /// Get a command by `name`. This can either be the command name or any of it's aliases.
    fn command(&self, name: String) -> Option<&Command> {
        let name = self.aliases.get(&name).unwrap_or(&name);
        self.commands.get(name)
    }

    /// Handle a privmsg
    pub async fn handle_privmsg(
        &self,
        msg: Arc<twitchchat::messages::Privmsg<'_>>,
        writer: &mut twitchchat::client::Writer,
    ) {
        let message = msg.data.trim().replace("\u{e0000}", ""); // remove chatterino chars
        let words: Vec<String> = message.split_whitespace().map(|s| s.to_string()).collect();
        let mut command_name = words[0].to_owned();
        let prefix = command_name.remove(0);

        if prefix != self.prefix {
            trace!("Dropping message because prefix was not found");
            return;
        }

        let args = &words[1..];
        trace!("Command: {} Args: {:?}", command_name, args);

        match self.command(command_name) {
            Some(cmd) => {
                debug!("Found matching command {}", cmd.name());
                if !cmd.whitelisted() {
                    // or the command is enabled in this channel

                    trace!("Executing command");
                    match cmd.execute(args.to_vec(), msg.clone()) {
                        CommandResult::Message(m) => {
                            writer
                                .privmsg(&msg.channel, &m)
                                .await
                                .expect("Could not write to channel");
                        }
                        CommandResult::NoMessage => (), // do nothing
                        CommandResult::Chainable(v) => {
                            //if is_chaining() {
                            //} else {

                            writer
                                .privmsg(&msg.channel, &v[0])
                                .await
                                .expect("Could not write to channel");
                        }
                        CommandResult::Error(e) => {
                            error!("Could not execute command (name: {}): {}", cmd.name(), e);
                        }
                    }
                }
            }
            None => debug!("No matching command found"),
        }
    }
}

pub fn new(context: Arc<Context>) -> CommandHandler {
    use super::befehle;
    let mut ch = CommandHandler::new(context.clone());

    ch.add_all(befehle::all(context.clone()));

    ch
}
