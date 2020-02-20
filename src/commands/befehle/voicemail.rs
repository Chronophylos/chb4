use super::prelude::*;

pub fn command(_context: Arc<Context>) -> Command {
    Command::with_name("voicemail")
        .aliases(vec![String::from("tell")])
        .command(|_args, _msg| CommandResult::Error(String::from("Not implemented")))
        .done()
}
