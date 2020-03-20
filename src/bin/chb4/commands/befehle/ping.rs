use super::prelude::*;
use chrono::prelude::*;
use humantime::format_duration;
use std::time::Duration;

pub fn command(context: Arc<Context>) -> Command {
    Command::with_name("ping")
        .chainable()
        .command(move |_args, msg| {
            let now = Utc::now().timestamp_millis() as u64;
            let ts = match msg.tmi_sent_ts() {
                Some(ts) => ts,
                None => return CommandResult::Error(String::from("Missing TMI_sent_TS")),
            };
            let latency = Duration::from_millis(now - ts);
            let elapsed = context.elapsed();

            CommandResult::Chainable(vec![
                format!(
                    "Pong! Latency to TMI: {}. The bot has been running for {}",
                    format_duration(latency),
                    format_duration(truncate_duration(elapsed))
                ),
                format!("{}", now - ts),
                format!("{}", elapsed.as_millis()),
            ])
        })
        .done()
}
