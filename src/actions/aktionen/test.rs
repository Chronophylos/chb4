use super::super::action::{Action, ActionResult};
use rand::prelude::*;
use regex::Regex;

pub fn action() -> Action {
    Action::with_name(String::from("test"))
        .regex(Regex::new(r"^test").unwrap())
        .command(|_| {
            let successful: bool = random();
            match successful {
                true => ActionResult::Message(String::from("Test successful ppHop")),
                false => ActionResult::Message(String::from("Test unsuccessful FeelsBadMan")),
            }
        })
        .done()
}
