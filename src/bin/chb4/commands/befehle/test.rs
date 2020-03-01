use super::prelude::*;

pub fn command() -> Command {
    Command::with_name("test")
        .command(|args, _msg| {
            if args.is_empty() {
                CommandResult::Message(String::from("Test what?"))
            } else {
                CommandResult::Message(String::from("Testing ") + &args[0])
            }
        })
        .description(
            "A Command to test the bot.

USAGE: test [TEXT]...
",
        )
        .done()
}
