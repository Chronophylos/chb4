use super::prelude::*;
use chb4::voicemail::Voicemail;
use chrono::prelude::*;
use humantime::format_duration;

lazy_static! {
    static ref SEPERATORS: Vec<&'static str> = vec!["&&", "and", "und"];
}

pub fn command(context: Arc<BotContext>) -> Command {
    Command::with_name("voicemail")
        .alias("tell")
        .command(move |args, msg, _user| {
            let user_id = msg.twitch_id().unwrap();
            let line = args.join(" ");
            let mut voicemail: Voicemail = match line.parse() {
                Ok(v) => v,
                Err(e) => return Ok(MessageResult::Message(format!("{:?}", e))),
            };

            let conn = &context.conn();

            let channel_name = msg.channel().to_owned();
            let channel = match database::Channel::by_name(conn, channel_name.trim_start_matches('#')) {
                Ok(c) => match c {
                    Some(c) => c,
                    None => return Err(MessageError::from(format!("Channel not in database (name: {})", msg.channel()))),
                },
                Err(e)=>return Err(MessageError::from(e.to_string())),
            };

            let bot_name = context.bot_name();
            voicemail.recipients.retain(|x| x != &bot_name);

            let now = Utc::now().naive_utc();
            match database::Voicemail::new(
                conn,
                &voicemail,
                user_id as i64,
                channel.id,
                now,
            ) {
                Ok(voicemails) => {
                    if voicemail.schedule.is_none() {
                        Ok(MessageResult::Message(format!(
                            "I'll send that message to {} when they next type in chat.",
                            voicemail.recipients.join(", ")
                        )))
                    } else {
                        // actually schedule voicemail

                        for voicemail in voicemails {
                            context.scheduler().schedule(voicemail);
                        }

                        Ok(MessageResult::Message(format!(
                            "I'll send that message to {} in {}",
                            voicemail.recipients.join(", "),
                            format_duration(
                                    voicemail.schedule
                                    .unwrap()
                                    .signed_duration_since(now)
                                    .to_std()
                                    .unwrap_or_default()
                            ),
                        )))
                    }
                }
                Err(e) => Err(MessageError::from(e.to_string())),
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
