use super::super::command::{Command, CommandResult};

pub fn command<'a>() -> Command<'a> {
    Command::with_name("test")
        .command(|args| match args.is_empty() {
            true => CommandResult::Message(String::from("Test what?")),
            false => CommandResult::Message(String::from("Testing ") + args[0]),
        })
        .done()
}
