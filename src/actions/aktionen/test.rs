use super::super::action::{Action, ActionResult};
use rand::prelude::*;

pub fn action<'a>() -> Action<'a> {
    Action::with_name("test")
        .regex(r"^test")
        .command(|_| {
            let successful: bool = random();
            if successful {
                ActionResult::Message("Test successful ppHop")
            } else {
                ActionResult::Message("Test unsuccessful FeelsBadMan")
            }
        })
        .done()
}
