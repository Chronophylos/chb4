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
mod time;
mod version;
mod voicemail;

pub fn all() -> Vec<Arc<Command>> {
    vec![
        admin::command(),
        color::command(),
        lastseen::command(),
        man::command(),
        math::command(),
        ping::command(),
        quote::command(),
        system::command(),
        test::command(),
        time::command(),
        version::command(),
        voicemail::command(),
    ]
}
