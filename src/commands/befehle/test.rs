use crate::commands::command::{Command, CommandResult};

pub fn command() -> Command {
    Command::with_name("test")
        .command(|args, _msg| {
            if args.is_empty() {
                CommandResult::Message(String::from("Test what?"))
            } else {
                CommandResult::Message(String::from("Testing ") + &args[0])
            }
        })
        .done()
}
