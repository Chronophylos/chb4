use super::prelude::*;

pub fn command() -> Arc<Command> {
    Command::with_name("test")
        .aliases(vec!["tset", "tets"])
        .command(|_context, args, _msg, _user| {
            Ok(MessageResult::Message(if args.is_empty() {
                "Test what?".into()
            } else {
                format!("Testing {}", &args.join(" "))
            }))
        })
        .about("Test everything!")
        .description(
            "
USAGE: test [TEXT]...
",
        )
        .done()
}
