use super::prelude::*;

lazy_static! {
    static ref SEPERATORS: Vec<&'static str> = vec!["&&", "and", "und"];
}

pub fn command(context: Arc<Context>) -> Command {
    Command::with_name("voicemail")
        .alias("tell")
        .command(move |args, msg| todo!())
        .done()
}
