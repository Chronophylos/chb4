use crate::command::{Command, CommandResult};
use crate::context::Context;
use std::sync::Arc;

pub fn command<'a>(context: Arc<Context>) -> Command<'a> {
    Command::with_name("voicemail")
        .aliases(vec!["tell"])
        .command(|bot, args| {})
        .done()
}
