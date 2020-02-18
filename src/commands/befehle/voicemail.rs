use super::super::command::{Command, CommandResult};

pub fn command<'a>() -> Command<'a> {
    Command::with_name("voicemail")
        .aliases(vec!["tell"])
        .command(|bot, args| {

        })
    .done()
}
