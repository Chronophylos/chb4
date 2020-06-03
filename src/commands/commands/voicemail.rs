use super::prelude::*;
use crate::voicemail::Voicemail;
use chrono::prelude::*;
use humantime::format_duration;

lazy_static! {
    static ref SEPERATORS: Vec<&'static str> = vec!["&&", "and", "und"];
}

pub fn command() -> Arc<Command> {
    Command::with_name("voicemail")
        .alias("tell")
        .command(move |context, args, msg, _user| {
            let user_id = msg.twitch_id().unwrap();
            let line = args.join(" ");
            let mut voicemail: Voicemail = match line.parse() {
                Ok(v) => v,
                Err(err) => {
                    return Ok(MessageResult::Error(format!(
                        "Could not parse voicemail: {}",
                        err
                    )))
                }
            };

            let conn = &context.conn();

            let channel_name = msg.channel().to_owned();
            let channel = database::Channel::by_name(conn, channel_name.trim_start_matches('#'))
                .context("Could not get channel from database")?
                .context("Channel is not in database")?;

            let bot_name = context.bot_name();
            voicemail.recipients.retain(|x| x != &bot_name);

            let now = Utc::now().naive_utc();
            let voicemails =
                database::Voicemail::new(conn, &voicemail, user_id as i64, channel.id, now)
                    .context("Could not insert voicemail(s) to database")?;
            if voicemail.schedule.is_none() {
                Ok(MessageResult::Message(format!(
                    "I'll send that message to {} when they next type in chat.",
                    voicemail.recipients.join(", ")
                )))
            } else {
                // actually schedule voicemail

                for voicemail in voicemails {
                    context.scheduler().schedule(voicemail).unwrap();
                }

                Ok(MessageResult::Message(format!(
                    "I'll send that message to {} in {}",
                    voicemail.recipients.join(", "),
                    format_duration(
                        voicemail
                            .schedule
                            .unwrap()
                            .signed_duration_since(now)
                            .to_std()
                            .unwrap_or_default()
                    ),
                )))
            }
        })
        .about("Send messages to other users or yourself")
        .description(
            "
=== USAGE

```
tell RECIPIENTS [SCHEDULE] MESSAGE
```

==== RECIPIENTS

A separated list of recipients.
Valid separators are `and`, `und`, `&&` and `,`.

.Example:
    nymn and pajlada

==== SCHEDLUE

To schedule a voicemail you need a marker and a value.

[cols=3*,options=header]
|===
| type
| markers
| value

| relative
| `in`
| A number and a unit, like `20 minutes` or `1 day`. You can even combine values: `1 day 12 hours`

| absolute
| `on`, `at`
| A https://tools.ietf.org/html/rfc3339[RFC3339] Date-Time formatted string.
RFC2822 and keywords like `noon` are wip.
|===

.Example:
    in 20 minutes 2 hours
    at 2020-02-20 20:20
",
        )
        .done()
}
