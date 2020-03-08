use super::prelude::*;

use futures_executor::block_on;

pub fn command(context: Arc<Context>) -> Command {
    Command::with_name("admin")
        .command(move |args: Vec<String>, msg: Arc<Privmsg<'_>>| {
            let permission = Permission::from(context.clone(), msg).unwrap();

            if permission != Permission::Owner {
                debug!("Permission not high enough (is {:?})", permission);
                return CommandResult::NoMessage;
            }

            match args.get(0).map(String::as_str) {
                Some("stop") => stop(context.clone()),
                Some("leave") => leave(context.clone(), args[1..].to_vec()),
                Some("join") => join(context.clone(), args[1..].to_vec()),
                Some(_) => CommandResult::Message(String::from("Unknown sub-command")),
                None => CommandResult::Message(String::from("Missing sub-command")),
            }
        })
        .description(
            "has various sub commands to do admin stuff.

USAGE: admin SUBCOMMAND

SUBCOMMANDS:
    stop - stop the bot
    leave CHANNEL - leave a channel
    join CHANNEL - join a channel
",
        )
        .done()
}

fn stop(context: Arc<Context>) -> CommandResult {
    warn!("Stopping bot by command!");

    // stop the chat client
    block_on(async {
        info!("Stopping chat client");
        context.chat().stop().await.unwrap()
    });

    info!("Stopping process");
    std::process::exit(0);
}

fn leave(context: Arc<Context>, args: Vec<String>) -> CommandResult {
    let channel = match args.get(0) {
        Some(c) => c,
        None => return CommandResult::Message(String::from("Missing channel")),
    };

    match database::channel::leave(&context.pool().get().unwrap(), &channel) {
        Err(e) => CommandResult::Error(format!("{:?}", e)),
        Ok(_) => CommandResult::Message(format!("Left {}", channel).to_string()),
    }
}

fn join(context: Arc<Context>, args: Vec<String>) -> CommandResult {
    let channel = match args.get(0) {
        Some(c) => c,
        None => return CommandResult::Message(String::from("Missing channel")),
    };

    match database::channel::join(&context.pool().get().unwrap(), &channel) {
        Err(e) => CommandResult::Error(format!("{:?}", e)),
        Ok(_) => CommandResult::Message(format!("Joined {}", channel).to_string()),
    }
}
