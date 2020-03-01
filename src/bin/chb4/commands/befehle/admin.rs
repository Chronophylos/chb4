use super::prelude::*;

use chb4::database;
use chb4::helpers::Permission;

pub fn command(context: Arc<Context>) -> Command {
    fn stop() -> CommandResult {
        use std::process;
        process::exit(0);
    }

    fn leave(context: Arc<Context>, args: Vec<String>) -> CommandResult {
        let channel = args.get(0).unwrap();

        match database::channel::leave(&context.pool().get().unwrap(), &channel) {
            Err(e) => CommandResult::Error(format!("{:?}", e)),
            Ok(_) => CommandResult::Message(format!("Left {}", channel).to_string()),
        }
    }

    fn join(context: Arc<Context>, args: Vec<String>) -> CommandResult {
        let channel = args.get(0).unwrap();

        match database::channel::join(&context.pool().get().unwrap(), &channel) {
            Err(e) => CommandResult::Error(format!("{:?}", e)),
            Ok(_) => CommandResult::Message(format!("Joined {}", channel).to_string()),
        }
    }

    Command::with_name("admin")
        .command(move |args: Vec<String>, msg: Arc<Privmsg<'_>>| {
            let permission = Permission::from(context.clone(), msg).unwrap();

            if permission != Permission::Owner {
                debug!("Permission not high enough");
                return CommandResult::NoMessage;
            }

            match args.get(0).map(String::as_str) {
                Some("stop") => stop(),
                Some("leave") => leave(context.clone(), args[1..].to_vec()),
                Some("join") => join(context.clone(), args[1..].to_vec()),
                Some(_) => CommandResult::Message(String::from("Unknown sub-command")),
                None => CommandResult::Message(String::from("Missing sub-command")),
            }
        })
        .done()
}
