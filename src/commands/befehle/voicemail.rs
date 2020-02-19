use crate::commands::command::{Command, CommandResult};
use crate::context::Context;
use std::sync::Arc;

pub fn command(_context: Arc<Context>) -> Command {
    Command::with_name("voicemail")
        .aliases(vec![String::from("tell")])
        .command(|_args, _msg| CommandResult::Error(String::from("Not implemented")))
        .done()
}
