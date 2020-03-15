use super::action::Action;
use chb4::context::Context;
use std::sync::Arc;

mod prelude;

mod test;
mod voicemail;

pub fn all(context: Arc<Context>) -> Vec<Action> {
    vec![test::action(), voicemail::action(context)]
}
