use super::super::action::{Action, ActionResult};
use rand::prelude::*;

pub fn action() -> Action {
    Action::with_name("test")
        .regex(r"^test")
        .command(|_| {
            let successful: bool = random();
            if successful {
                ActionResult::Message(String::from("Test successful ppHop"))
            } else {
                ActionResult::Message(String::from("Test unsuccessful FeelsBadMan"))
            }
        })
        .done()
}
