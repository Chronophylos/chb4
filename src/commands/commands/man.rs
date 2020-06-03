use super::prelude::*;

pub fn command() -> Arc<Command> {
    Command::with_name("man")
        .aliases(vec!["help", "whatis", "hilbe"])
        .command(move |context, args, _msg, _user| {
            if args.is_empty() {
                return Ok(MessageResult::MissingArgument("page"));
            }

            let (chapter, name) = match args.get(1) {
                Some(name) => (args.get(0), name),
                None => (None, args.get(0).unwrap()),
            };

            let chapter = chapter.cloned().map(|c| c.into());

            match context.whatis(chapter, name.to_owned()) {
                Some(m) => Ok(MessageResult::Message(m.short())),
                None => Ok(MessageResult::Message("No page found".into())),
            }
        })
        .about("Get help about a command")
        .description("
Provides manuals to every command or action. The manuals are split into different chapters.
Chapter `action` contains Actions such as the `voicemail (1)` action that allows replaying of messages.
Chapter `command` contains Commands like this `man (2)` command or the `voicemail (2)` command.

This command gives a short overview over a command. The full manual is available on the web.

USAGE: man [CHAPTER] PAGE
")
.example("
```
< man action voicemail
> voicemail - redeems voicemails created with tell https://crate.chronophylos.com/action/voicemail
```
")
        .done()
}
