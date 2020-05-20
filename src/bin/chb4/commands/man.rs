use super::prelude::*;

pub fn command() -> Arc<Command> {
    Command::with_name("man")
        .aliases(vec!["help", "whatis", "hilbe"])
        .command(move |context, args, _msg, _user| {
            if args.is_empty() {
                return Err(MessageError::from("Not enough arguments"));
            }

            let (chapter, name) = match args.get(1) {
                Some(name) => (args.get(0), name),
                None => (None, args.get(0).unwrap()),
            };

            let chapter = chapter.cloned().map(|c| c.into());

            match context.whatis(chapter, name.to_owned()) {
                Some(m) => Ok(MessageResult::Message(m.short())),
                None => Ok(MessageResult::Message(String::from("No page found"))),
            }
        })
        .about("Get help about a command")
        .done()
}
