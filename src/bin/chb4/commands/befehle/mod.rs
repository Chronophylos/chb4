use super::command::Command;
use chb4::context::Context;
use std::sync::Arc;

mod prelude;

// commands
mod admin;
mod color;
//mod font;
mod test;
mod voicemail;

pub fn all(context: Arc<Context>) -> Vec<Command> {
    vec![
        admin::command(context.clone()),
        color::command(),
        //font::command(),
        test::command(),
        //voicemail::command(context.clone()),
    ]
}
