use super::prelude::*;
use crate::database::Channel;

use futures_executor::block_on;

pub fn command() -> Arc<Command> {
    Command::with_name("admin")
        .command(move |context, args, msg, user| {
            let permission = Permission::from_user(msg, user).unwrap();

            if permission != Permission::Owner {
                debug!(
                    "Permission not high enough (is: {:?}, needed: {:?})",
                    permission,
                    Permission::Owner
                );
                return Ok(MessageResult::None);
            }

            match args.get(0).map(String::as_str) {
                Some("stop") => stop(context.clone()),
                Some("leave") => leave(context.clone(), args[1..].to_vec()),
                Some("join") => join(context.clone(), args[1..].to_vec()),
                Some(_) => Ok(MessageResult::Message("Unknown sub-command".into())),
                None => Ok(MessageResult::MissingArgument("Missing sub-command")),
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

fn stop(context: Arc<BotContext>) -> Result<MessageResult> {
    warn!("Stopping bot by command!");

    // stop the chat client
    block_on(async {
        info!("Stopping chat client");
        context.twitchbot().stop()
    });

    info!("Stopping process");
    std::process::exit(0);
}

fn leave(context: Arc<BotContext>, args: Vec<String>) -> Result<MessageResult> {
    let name = match args.get(0) {
        Some(c) => c,
        None => return Ok(MessageResult::MissingArgument("channel")),
    };

    let conn = &context.conn();
    let channel = Channel::by_name(conn, name)?;

    let channel = match channel {
        Some(c) => c,
        None => return Ok(MessageResult::Error(format!("I am no in channel {}", name))),
    };

    context.twitchbot().part(name)?;

    channel.leave(conn)?;

    Ok(MessageResult::Message(format!("I lef channel {}", name)))
}

fn join(context: Arc<BotContext>, args: Vec<String>) -> Result<MessageResult> {
    let channel = match args.get(0) {
        Some(c) => c,
        None => return Ok(MessageResult::MissingArgument("channel")),
    };

    let conn = &context.conn();

    context.twitchbot().join(channel)?;

    Channel::join(conn, &channel)?;

    Ok(MessageResult::Message(format!("Joined {}", channel)))
}
