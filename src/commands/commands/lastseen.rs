use super::prelude::*;
use crate::database::User;
use chrono::prelude::*;

pub fn command() -> Arc<Command> {
    Command::with_name("lastseen")
        .aliases(vec!["ls"])
        .command(|context, args, _msg, _user| {
            let name = match args.get(0) {
                None => return Ok(MessageResult::MissingArgument("name")),
                Some(name) => name,
            };

            let conn = context.conn();

            let user = User::by_name(&conn, &name.to_lowercase())?;
            match user {
                Some(user) => {
                    let now = Utc::now().naive_utc();
                    let last_seen = user.last_seen.context("last_seen is not set")?;
                    let duration = now.signed_duration_since(last_seen).to_std()?;

                    Ok(MessageResult::Message(format!(
                        "The user {} was last seen {} ago",
                        user.display_name_or_name(),
                        humantime::format_duration(truncate_duration(duration)),
                    )))
                }
                None => Ok(MessageResult::Message(
                    "I have never seen this user before".into(),
                )),
            }
        })
        .about("Check when a user was last seen")
        .done()
}
