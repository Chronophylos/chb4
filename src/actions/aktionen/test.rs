use super::super::action::{Action, ActionResult};
use rand::prelude::*;

pub fn action<'a>() -> Action<'a> {
    Action::with_name("test")
        .regex(r"^test")
        .command(|_| {
            let successful: bool = random();
            match successful {
                true => ActionResult::Message("Test successful ppHop"),
                false => ActionResult::Message("Test unsuccessful FeelsBadMan"),
            }
        })
        .done()
}
