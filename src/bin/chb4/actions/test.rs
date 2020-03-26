use super::prelude::*;
use rand::prelude::*;

pub fn action() -> Arc<Action> {
    Action::with_name("test")
        .regex(r"^test")
        .command(|_msg, _user| {
            Ok(MessageResult::Message(String::from(if random() {
                "Test successful ppHop"
            } else {
                "Test unsuccessful FeelsBadMan"
            })))
        })
        .done()
}
