use super::prelude::*;
use chrono::prelude::*;
use humantime::format_duration;
use std::time::Duration;

pub fn command() -> Arc<Command> {
    Command::with_name("ping")
        .chainable()
        .command(move |context, _args, msg, _user| {
            let now = Utc::now().timestamp_millis() as u64;
            let ts = msg.sent_ts();
            let latency = Duration::from_millis(now - ts);
            let elapsed = context.elapsed();

            Ok(MessageResult::Message(format!(
                "Pong! Latency to TMI: {}. The bot has been running for {}",
                format_duration(latency),
                format_duration(truncate_duration(elapsed))
            )))
        })
        .about("Get information about the bot instance")
        .done()
}
