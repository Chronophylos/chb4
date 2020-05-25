use super::prelude::*;

pub fn command() -> Arc<Command> {
    Command::with_name("color")
        .chainable()
        .command(|_context, _args, msg, _user| {
            let color = msg.color();
            Ok(MessageResult::ReplyWithValues(color.clone(), vec![color]))
        })
        .about("Print your current chat color")
        .description(
            "
This always prints the hex code.
Returning the actual name of the color is wip an depends on the `twitchchat` crate.
",
        )
        .example(
            "
```
> ~color
< Chronophylos, #7700B3
```",
        )
        .done()
}
