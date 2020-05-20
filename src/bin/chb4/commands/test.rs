use super::prelude::*;

pub fn command() -> Arc<Command> {
    Command::with_name("test")
        .aliases(vec!["tset", "tets"])
        .command(|_context, args, _msg, _user| {
            Ok(if args.is_empty() {
                MessageResult::Message(String::from("Test what?"))
            } else {
                MessageResult::Message(String::from("Testing ") + &args[0])
            })
        })
        .description(
            "A Command to test the bot.

USAGE: test [TEXT]...
",
        )
        .done()
}
