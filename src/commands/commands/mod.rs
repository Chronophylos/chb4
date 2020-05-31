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
mod system;
mod test;
mod version;
mod voicemail;

pub fn all() -> Vec<Arc<Command>> {
    vec![
        system::command(),
        version::command(),
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
