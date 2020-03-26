use chb4::{actions::action::Action, context::BotContext};
use std::sync::Arc;

mod prelude;

mod test;
mod voicemail;

pub fn all(context: Arc<BotContext>) -> Vec<Arc<Action>> {
    vec![test::action(), voicemail::action(context)]
}
