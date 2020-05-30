//! Everything needed to handle and create commands
use super::command::Command;
use crate::{
    context::BotContext,
    database::User,
    handler::{Handler, SimpleHandler, Twitch},
    message::{Message, MessageConsumer, MessageResult},
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use twitchchat::messages::Privmsg;

#[derive(Error, Debug)]
pub enum CommandHandlerError {
    #[error("Could not execute command (name: {0})")]
    ExecuteCommand(String),

    #[error("Could not send privmsg")]
    SendPrivmsg,
}

pub struct CommandHandler {
    commands: HashMap<String, Arc<Command>>,
    // translate aliases to command names
    aliases: HashMap<String, String>,

    /// The prefix to use when checking for commands in a message.
    prefix: char,

    #[allow(dead_code)]
    context: Arc<BotContext>,
}

impl CommandHandler {
    /// Create a new CommandHandler
    pub fn new(context: Arc<BotContext>, commands: Vec<Arc<Command>>) -> Self {
        let mut aliases: HashMap<String, String> = HashMap::new();
        let mut command_map: HashMap<String, Arc<Command>> = HashMap::new();

        for command in commands {
            for alias in command.aliases() {
                aliases.insert(alias.to_owned(), command.name().to_string());
            }
            command_map.insert(command.name().to_owned(), command);
        }

        Self {
            context,
            commands: command_map,
            aliases,
            prefix: '~',
        }
    }
}

impl SimpleHandler for CommandHandler {
    fn name(&self) -> &str {
        "command"
    }
}

impl Handler<Command> for CommandHandler {
    /// Get a command by `name`. This can either be the command name or any of it's aliases.
    fn get(&self, name: String) -> Option<Arc<Command>> {
        let name = self.aliases.get(&name).unwrap_or_else(|| &name);
        self.commands.get(name).cloned()
    }
}

#[async_trait]
impl Twitch for CommandHandler {
    async fn handle(&self, msg: Arc<Privmsg<'_>>, user: &User) -> Result<()> {
        let message = msg.data.trim().replace("\u{e0000}", ""); // remove chatterino chars
        let words: Vec<String> = message.split_whitespace().map(|s| s.to_string()).collect();
        let mut command_name = words[0].clone();
        let prefix = command_name.remove(0);

        if prefix != self.prefix {
            trace!("Prefix not found");
            return Ok(());
        }

        let args = &words[1..];
        trace!("Command: {} Args: {:?}", command_name, args);

        let cmd = match self.get(command_name) {
            Some(c) => c,
            None => {
                trace!("No matching command found");
                return Ok(());
            }
        };

        debug!("Found matching command {}", Command::name(&cmd));

        if cmd.whitelisted() {
            debug!(
                "Command is not whitelisted in this channel (name: {}, channel: {})",
                cmd.name(),
                msg.channel
            );
            return Ok(());
        }

        // or the command is enabled in this channel

        let mut writer = self.context.twitchbot().writer();

        trace!("Executing command");

        match cmd
            .consume(
                self.context.clone(),
                args.to_vec(),
                Message::TwitchPrivmsg(msg.clone()),
                user,
            )
            .context(CommandHandlerError::ExecuteCommand(cmd.name().to_owned()))?
        {
            MessageResult::None => Ok(()),
            MessageResult::Message(m) => writer
                .privmsg(&msg.channel, &m)
                .await
                .context(CommandHandlerError::SendPrivmsg),
            MessageResult::MessageWithValues(m, _v) => {
                //if is_chaining() {
                //} else {

                writer
                    .privmsg(&msg.channel, &m)
                    .await
                    .context(CommandHandlerError::SendPrivmsg)
            }
            MessageResult::Reply(m) => writer
                .privmsg(
                    &msg.channel,
                    format!("{}, {}", user.display_name_or_name(), m).as_str(),
                )
                .await
                .context(CommandHandlerError::SendPrivmsg),
            MessageResult::ReplyWithValues(m, _v) => writer
                .privmsg(
                    &msg.channel,
                    format!("{}, {}", user.display_name_or_name(), m).as_str(),
                )
                .await
                .context(CommandHandlerError::SendPrivmsg),
        }
    }
}
