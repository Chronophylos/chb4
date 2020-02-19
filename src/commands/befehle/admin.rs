use crate::commands::command::{Command, CommandResult};
use crate::context::Context;
use std::sync::Arc;
use twitchchat::messages::Privmsg;

use crate::helpers::Permission;

pub fn command(context: Arc<Context>) -> Command {
    fn stop(_args: Vec<String>) -> CommandResult {
        use std::process;
        process::exit(0);
    }
    fn leave(_args: Vec<String>) -> CommandResult {
        CommandResult::Error(String::from("Not implemented"))
    }
    fn join(_args: Vec<String>) -> CommandResult {
        CommandResult::Error(String::from("Not implemented"))
    }

    Command::with_name("admin")
        .command(move |args: Vec<String>, msg: Arc<Privmsg<'_>>| {
            let permission = Permission::from(context.clone(), msg);

            if permission != Permission::Owner {
                return CommandResult::NoMessage;
            }

            match args.get(0).map(String::as_str) {
                Some("stop") => stop(args[1..].to_vec()),
                Some("leave") => leave(args[1..].to_vec()),
                Some("join") => join(args[1..].to_vec()),
                Some(_) => CommandResult::Message(String::from("Unknown sub-command")),
                None => CommandResult::Message(String::from("Missing sub-command")),
            }
        })
        .done()
}
