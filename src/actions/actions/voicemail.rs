use super::prelude::*;
use crate::database::Voicemail;

pub fn action() -> Arc<Action> {
    Action::with_name("voicemail")
        .command(move |context, _msg, user| {
            let conn = &context.conn();

            let voicemails = user.pop(conn).context("Could not pop voicemails")?;

            if voicemails.is_empty() {
                trace!("No voicemails found");
                return Ok(MessageResult::None);
            }

            trace!("Found {} voicemails", voicemails.len());

            Ok(MessageResult::Message(Voicemail::format_vec(
                conn, voicemails,
            )?))
        })
        .noisy()
        .done()
}
