use super::command::Command;
use crate::context::Context;
use std::sync::Arc;

mod admin;
mod test;
mod voicemail;

pub fn all(context: Arc<Context>) -> Vec<Command> {
    vec![
        test::command(),
        admin::command(context.clone()),
        //voicemail::command(context.clone()),
    ]
}
