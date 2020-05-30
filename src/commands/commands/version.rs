use super::prelude::*;

pub fn command() -> Arc<Command> {
    Command::with_name("version")
        .command(|context, _args, _msg, _user| {
            Ok(MessageResult::Message(format!(
                "Currently running CHB4 Version {} ({})",
                context.version, context.git_commit,
            )))
        })
        .about("Get the current version")
        .done()
}
