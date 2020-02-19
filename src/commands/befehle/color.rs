use crate::commands::command::{Command, CommandResult};

pub fn command() -> Command {
    Command::with_name("color")
        .command(|_args, msg| {
            let color = msg.color().unwrap();
            CommandResult::Message(format!("{}", color))
        })
        .done()
}
