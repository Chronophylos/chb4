use crate::commands::command::Command;
use std::sync::Arc;

mod prelude;

mod admin;
mod color;
mod lastseen;
mod man;
mod math;
mod ping;
mod quote;
mod test;
mod voicemail;

pub fn all() -> Vec<Arc<Command>> {
    vec![
        admin::command(),
        color::command(),
        man::command(),
        math::command(),
        ping::command(),
        quote::command(),
        test::command(),
        lastseen::command(),
        voicemail::command(),
    ]
}
