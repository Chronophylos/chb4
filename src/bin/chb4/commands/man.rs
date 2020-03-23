use super::prelude::*;

fn command() {
    Command::with_name("man")
        .command(|args, msg| {
            lazy_static! {
                static ref COMMANDS: Vec<Command> = super::commands();
            }
        })
        .done()
}
