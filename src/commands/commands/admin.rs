use super::prelude::*;
use crate::database::Channel;

use futures_executor::block_on;

pub fn command() -> Arc<Command> {
    Command::with_name("admin")
        .command(move |context, args, msg, user| {
            let permission = Permission::from_user(msg, user).unwrap();

            if permission != Permission::Owner {
                debug!("Permission not high enough (is {:?})", permission);
                return Ok(MessageResult::None);
            }

            match args.get(0).map(String::as_str) {
                Some("stop") => stop(context.clone()),
                Some("leave") => leave(context.clone(), args[1..].to_vec()),
                Some("join") => join(context.clone(), args[1..].to_vec()),
                Some(_) => Ok(MessageResult::Message(String::from("Unknown sub-command"))),
                None => Ok(MessageResult::Message(String::from("Missing sub-command"))),
            }
        })
        .about("Various commands to manage the bot.")
        .description(
            r#"
NOTE: This is a owner only command!

=== USAGE

```
admin SUBCOMMAND
```

.SUBCOMMAND
* `stop` -- stop the bot
* `leave CHANNEL` -- leave a channel
* `join CHANNEL` -- join a channel
"#,
        )
        .done()
}

fn stop(context: Arc<BotContext>) -> Result {
    warn!("Stopping bot by command!");

    // stop the chat client
    block_on(async {
        info!("Stopping chat client");
        context.twitchbot().stop()
    });

    info!("Stopping process");
    std::process::exit(0);
}

fn leave(context: Arc<BotContext>, args: Vec<String>) -> Result {
    let name = match args.get(0) {
        Some(c) => c,
        None => return Ok(MessageResult::Message(String::from("Missing channel"))),
    };

    let conn = &context.conn();
    let channel = match Channel::by_name(conn, name) {
        Ok(c) => c,
        Err(e) => return Err(MessageError::from(e.to_string())),
    };

    let channel = match channel {
        Some(c) => c,
        None => return Ok(MessageResult::Message(String::from("Channel not found"))),
    };

    if let Err(e) = context.twitchbot().part(name) {
        return Err(MessageError::from(e.to_string()));
    }

    match channel.leave(conn) {
        Err(e) => Err(MessageError::from(e.to_string())),
        Ok(_) => Ok(MessageResult::Message(format!("left channel {}", name))),
    }
}

fn join(context: Arc<BotContext>, args: Vec<String>) -> Result {
    let channel = match args.get(0) {
        Some(c) => c,
        None => return Ok(MessageResult::Message(String::from("Missing channel"))),
    };

    let conn = &context.conn();

    if let Err(e) = context.twitchbot().join(channel) {
        return Err(MessageError::from(e.to_string()));
    }

    match Channel::join(conn, &channel) {
        Err(e) => Err(MessageError::from(e.to_string())),
        Ok(_) => Ok(MessageResult::Message(format!("Joined {}", channel))),
    }
}
