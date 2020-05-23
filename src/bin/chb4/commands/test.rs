use super::prelude::*;

pub fn command() -> Arc<Command> {
    Command::with_name("test")
        .aliases(vec!["tset", "tets"])
        .command(|_context, args, _msg, _user| {
            Ok(if args.is_empty() {
                MessageResult::Message(String::from("Test what?"))
            } else {
                MessageResult::Message(String::from("Testing ") + &args.join(" "))
            })
        })
        .about("Test everything!")
        .description(
            "
USAGE: test [TEXT]...
",
        )
        .done()
}
