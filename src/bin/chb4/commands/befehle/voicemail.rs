use super::prelude::*;
use chb4::voicemail::Voicemail;
use chrono::prelude::*;

lazy_static! {
    static ref SEPERATORS: Vec<&'static str> = vec!["&&", "and", "und"];
}

pub fn command(context: Arc<Context>) -> Command {
    Command::with_name("voicemail")
        .alias("tell")
        .command(move |args, msg| {
            let user_id = msg.user_id().unwrap() as i32;
            let line = args.join(" ");
            let voicemail: Voicemail = match line.parse() {
                Ok(v) => v,
                Err(e) => return CommandResult::Message(format!("{:?}", e)),
            };

            match database::voicemail::new(
                &context.pool().get().unwrap(),
                &voicemail,
                user_id,
                Utc::now().naive_utc(),
            ) {
                Ok(_) => {
                    if voicemail.schedule.is_none() {
                        CommandResult::Message(format!(
                            "I'll send that message to {} when they next type in chat.",
                            voicemail.recipients.join(", ")
                        ))
                    } else {
                        CommandResult::Message(format!(
                            "I'll send that message to {} on {}",
                            voicemail.recipients.join(", "),
                            voicemail.schedule.unwrap()
                        ))
                    }
                }
                Err(e) => CommandResult::Error(e.to_string()),
            }
        })
        .done()
}
