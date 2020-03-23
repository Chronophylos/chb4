use super::prelude::*;
use chb4::database::Voicemail;

pub fn action(context: Arc<BotContext>) -> Action {
    Action::with_name("voicemail")
        .command(move |_msg, user| {
            let conn = &context.conn();

            let voicemails = match user.pop(conn) {
                Ok(v) => v,
                Err(e) => return Err(MessageError::from(e.to_string())),
            };

            if voicemails.is_empty() {
                trace!("No voicemails found");
                return Ok(MessageResult::None);
            }

            trace!("Found {} voicemails", voicemails.len());

            match Voicemail::format_vec(conn, voicemails) {
                Ok(m) => Ok(MessageResult::Message(m)),
                Err(e) => Err(MessageError::from(e.to_string())),
            }
        })
        .done()
}
