use chb4::{commands::command::Command, context::BotContext};
use std::sync::Arc;

mod prelude;

mod admin;
mod color;
mod man;
mod math;
mod ping;
mod quote;
mod test;
mod voicemail;

pub fn all(context: Arc<BotContext>) -> Vec<Arc<Command>> {
    vec![
        admin::command(context.clone()),
        color::command(),
        man::command(context.clone()),
        math::command(),
        ping::command(context.clone()),
        quote::command(context.clone()),
        test::command(),
        voicemail::command(context),
    ]
}
