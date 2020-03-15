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
            let user_id = msg.user_id().unwrap();
            let line = args.join(" ");
            let voicemail: Voicemail = match line.parse() {
                Ok(v) => v,
                Err(e) => return CommandResult::Message(format!("{:?}", e)),
            };

            match database::voicemail::new(
                &context.conn(),
                &voicemail,
                user_id as i64,
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
        .description(
            "send other users a message.

USAGE: tell RECIPIENTS [SCHEDULE] MESSAGE

RECIPIENTS:
    A separated list of recipients. Valid separators are `and`, `und`, `&&` and `,`.

SCHEDLUE:
    Either relative or absolute. 
    * A relative schedule is something like `in 20 minutes` and requires a `in` before any times.
    * A absoulte schedule is something like `on 2020-03-11` and requires a `at` or `on` befor the time.
",
        )
        .done()
}
