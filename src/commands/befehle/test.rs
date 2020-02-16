use super::super::command::{Command, CommandResult};

pub fn command<'a>() -> Command<'a> {
    Command::with_name("test")
        .command(|args| {
            if args.is_empty() {
                CommandResult::Message(String::from("Test what?"))
            } else {
                CommandResult::Message(String::from("Testing ") + args[0])
            }
        })
        .done()
}
