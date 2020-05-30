use super::prelude::*;
use crate::database::User;
use chrono::prelude::*;

pub fn command() -> Arc<Command> {
    Command::with_name("lastseen")
        .aliases(vec!["ls"])
        .command(|context, args, _msg, _user| {
            let name = match args.get(0) {
                None => {
                    return Ok(MessageResult::Message(String::from(
                        "Missing argument name",
                    )))
                }
                Some(name) => name,
            };

            let conn = context.conn();

            match User::by_name(&conn, &name.to_lowercase()) {
                Ok(user) => match user {
                    Some(user) => {
                        let now = Utc::now().naive_utc();
                        let last_seen = match user.last_seen {
                            Some(t) => t,
                            None => return Err(MessageError::from("last_seen is null")),
                        };

                        let duration = match now.signed_duration_since(last_seen).to_std() {
                            Ok(d) => d,
                            Err(err) => return Err(MessageError::from(err.to_string())),
                        };

                        Ok(MessageResult::MessageWithValues(
                            format!(
                                "The user {} was last seen {} ago",
                                user.display_name_or_name(),
                                humantime::format_duration(truncate_duration(duration)),
                            ),
                            vec![last_seen.format("%Y-%m-%d %H:%M:%S").to_string()],
                        ))
                    }
                    None => Ok(MessageResult::Message(String::from(
                        "I have never seen this user before",
                    ))),
                },
                Err(err) => Err(MessageError::from(err.to_string())),
            }
        })
        .about("Check when a user was last seen")
        .done()
}
