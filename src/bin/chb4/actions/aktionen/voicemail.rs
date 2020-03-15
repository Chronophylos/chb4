use super::prelude::*;
use chb4::database::{user, voicemail};
use chb4::models::Voicemail;
use snafu::{OptionExt, ResultExt, Snafu};

#[derive(Snafu, Debug)]
enum Error {
    #[snafu(display("Getting receiver: {}", source))]
    GetReceiver { source: user::Error },

    #[snafu(display("Receiver not found (id: {})", id))]
    ReceiverNotFound { id: i32 },

    #[snafu(display("Getting creator: {}", source))]
    GetCreator { source: user::Error },

    #[snafu(display("Creator not found (id: {})", id))]
    CreatorNotFound { id: i32 },
}

type Result<T> = std::result::Result<T, Error>;

pub fn action(context: Arc<Context>) -> Action {
    Action::with_name("voicemail")
        .command(move |msg| {
            let user_id = msg.user_id().unwrap() as i32;
            let voicemails = match voicemail::pop(&context.conn(), user_id) {
                Ok(v) => v,
                Err(e) => return ActionResult::Error(e.to_string()),
            };

            if voicemails.is_empty() {
                return ActionResult::NoMessage;
            }

            match format_voicemails(context.clone(), voicemails) {
                Ok(m) => ActionResult::Message(m),
                Err(e) => ActionResult::Error(e.to_string()),
            }
        })
        .done()
}

fn format_voicemails(context: Arc<Context>, voicemails: Vec<Voicemail>) -> Result<String> {
    let receiver_id = voicemails[0].receiver_id;
    let receiver = user::by_id(&context.conn(), receiver_id)
        .context(GetReceiver)?
        .context(ReceiverNotFound { id: receiver_id })?;

    let voicemails: Vec<_> = voicemails
        .iter()
        .map(|v| format_voicemail(context.clone(), v))
        .collect::<Result<Vec<_>>>()?;

    Ok(format!(
        "{}, {} messages for you: {}",
        receiver.display_name_or_name(),
        voicemails.len(),
        voicemails.join(" – ")
    ))
}

fn format_voicemail(context: Arc<Context>, voicemail: &Voicemail) -> Result<String> {
    let creator = user::by_id(&context.conn(), voicemail.creator_id)
        .context(GetCreator)?
        .context(CreatorNotFound {
            id: voicemail.creator_id,
        })?;

    Ok(format!(
        "{}, {}: {}",
        creator.display_name_or_name(),
        voicemail.created,
        voicemail.message
    ))
}
