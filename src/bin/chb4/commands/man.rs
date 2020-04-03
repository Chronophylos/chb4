use super::prelude::*;

pub fn command(context: Arc<BotContext>) -> Arc<Command> {
    Command::with_name("man")
        .command(move |args, _msg, _user| {
            if args.is_empty() {
                return Err(MessageError::from("Not enough arguments"));
            }

            let (chapter, name) = match args.get(1) {
                Some(name) => (args.get(0), name),
                None => (None, args.get(0).unwrap()),
            };

            let chapter = chapter.cloned().map(|c| c.into());

            match context.whatis(chapter, name.to_owned()) {
                Some(m) => Ok(MessageResult::Message(m.to_string())),
                None => Ok(MessageResult::Message(String::from("No page found"))),
            }
        })
        .done()
}
