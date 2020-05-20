use super::prelude::*;

pub fn command() -> Arc<Command> {
    Command::with_name("color")
        .chainable()
        .command(|_context, _args, msg, _user| {
            let color = msg.color();
            Ok(MessageResult::ReplyWithValues(color.clone(), vec![color]))
        })
        .about("print your current chat color")
        .description("Prints your chat color")
        .done()
}
