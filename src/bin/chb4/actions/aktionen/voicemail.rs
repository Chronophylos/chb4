use super::prelude::*;
use chb4::database::{User, Voicemail};

pub fn action(context: Arc<Context>) -> Action {
    Action::with_name("voicemail")
        .command(move |msg| {
            let user_id = msg.user_id().unwrap() as i64;
            let conn = &context.conn();
            let user = match User::by_twitch_id(conn, user_id) {
                Ok(u) => match u {
                    Some(u) => u,
                    None => return ActionResult::Error(String::from("User not found")),
                },
                Err(e) => return ActionResult::Error(e.to_string()),
            };

            let voicemails = match user.pop(conn) {
                Ok(v) => v,
                Err(e) => return ActionResult::Error(e.to_string()),
            };

            if voicemails.is_empty() {
                trace!("No voicemails found");
                return ActionResult::NoMessage;
            }

            trace!("Found {} voicemails", voicemails.len());

            match Voicemail::format_vec(conn, voicemails) {
                Ok(m) => ActionResult::Message(m),
                Err(e) => ActionResult::Error(e.to_string()),
            }
        })
        .done()
}
