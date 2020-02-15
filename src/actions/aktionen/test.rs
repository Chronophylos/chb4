use super::super::action::Action;
use rand::prelude::*;
use regex::Regex;

pub fn action() -> Action {
    Action::with_name(String::from("test"))
        .regex(Regex::new(r"^test").unwrap())
        .command(|msg, writer| {
            let successful: bool = random();
            let message = match successful {
                true => "Test successful ppHop",
                false => "Test unsuccessful FeelsBadMan",
            };
            writer.privmsg(&msg.channel, message);
        })
        .done()
}
