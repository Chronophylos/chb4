use super::prelude::*;
use chb4::database::{user, voicemail};
use chb4::models::Voicemail;

pub fn action(context: Arc<Context>) -> Action {
    Action::with_name("voicemail")
        .command(move |msg| {
            let user_id = msg.user_id().unwrap() as i64;
            let voicemails = match voicemail::pop(&context.conn(), user_id) {
                Ok(v) => v,
                Err(e) => return ActionResult::Error(e.to_string()),
            };

            if voicemails.is_empty() {
                trace!("No voicemails found");
                return ActionResult::NoMessage;
            }

            trace!("Found {} voicemails", voicemails.len());

            match format_voicemails(context.clone(), voicemails) {
                Ok(m) => ActionResult::Message(m),
                Err(e) => ActionResult::Error(e.to_string()),
            }
        })
        .done()
}
