use super::prelude::*;

pub fn command() -> Command {
    Command::with_name("color")
        .chainable()
        .command(|_args, msg| {
            let color = msg.color().unwrap();
            CommandResult::Chainable(vec![
                format!("Your color is {}", color),
                format!("{}", color),
            ])
        })
        .description("Prints your chat color")
        .done()
}
