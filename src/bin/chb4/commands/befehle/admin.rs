use super::prelude::*;
use chb4::database::Channel;

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

This is a owner only command!

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
    let name = match args.get(0) {
        Some(c) => c,
        None => return CommandResult::Message(String::from("Missing channel")),
    };

    let conn = &context.conn();
    let channel = match Channel::by_name(conn, name) {
        Ok(c) => c,
        Err(e) => return CommandResult::Error(e.to_string()),
    };

    let channel = match channel {
        Some(c) => c,
        None => return CommandResult::Message(String::from("Channel not found")),
    };

    context.join_channel_sync(name.to_owned());

    match channel.leave(conn) {
        Err(e) => CommandResult::Error(format!("{:?}", e)),
        Ok(_) => CommandResult::Message(format!("left channel {}", name)),
    }
}

fn join(context: Arc<Context>, args: Vec<String>) -> CommandResult {
    let channel = match args.get(0) {
        Some(c) => c,
        None => return CommandResult::Message(String::from("Missing channel")),
    };

    let conn = &context.conn();

    context.join_channel_sync(channel.to_owned());

    match Channel::join(conn, &channel) {
        Err(e) => CommandResult::Error(format!("{:?}", e)),
        Ok(_) => CommandResult::Message(format!("Joined {}", channel)),
    }
}
