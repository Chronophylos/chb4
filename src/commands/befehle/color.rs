use super::prelude::*;

pub fn command() -> Command {
    Command::with_name("color")
        .command(|_args, msg| {
            let color = msg.color().unwrap();
            CommandResult::Message(format!("{}", color))
        })
        .done()
}
