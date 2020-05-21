use crate::{
    context::BotContext,
    database::{self, Channel, User, Voicemail},
};
use chrono::prelude::*;
use futures_delay_queue::{delay_queue, DelayQueue, Receiver};
use snafu::{OptionExt, ResultExt, Snafu};
use std::sync::Arc;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Scheduled is not set"))]
    GetScheduled,

    #[snafu(display("Converting to std duration: {}", source))]
    ConvertToStdDuration { source: time::OutOfRangeError },

    #[snafu(display("Getting voicemail: {}", source))]
    GetVoicemail { source: database::voicemail::Error },

    #[snafu(display("Voicemail not found (id: {})", id))]
    VoicemailNotFound { id: i32 },

    #[snafu(display("Getting channel: {}", source))]
    GetChannel { source: database::channel::Error },

    #[snafu(display("Channel not found (id: {})", id))]
    ChannelNotFound { id: i32 },

    #[snafu(display("Getting channel name: {}", source))]
    GetChannelName { source: database::channel::Error },

    #[snafu(display("Getting user: {}", source))]
    GetUser { source: database::user::Error },

    #[snafu(display("User not found (id: {})", id))]
    UserNotFound { id: i32 },

    #[snafu(display("Disabling voicemail: {}", source))]
    DisableVoicemail { source: database::voicemail::Error },

    #[snafu(display("Sending privmsg (channel: {}): {}", channel, source))]
    SendPrivmsg {
        source: twitchchat::Error,
        channel: String,
    },
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct Scheduler {
    queue: DelayQueue<i32>,
    receiver: Receiver<i32>,
}

impl Scheduler {
    pub fn new() -> Self {
        // create a queue with maximum size
        let (queue, receiver) = delay_queue(100_000_000);
        Self { queue, receiver }
    }

    pub async fn run(&self, context: Arc<BotContext>) -> Result<()> {
        debug!("starting scheduler loop");

        loop {
            if let Some(id) = self.receiver.receive().await {
                self.show(id, context.clone()).await?;
            }
        }
    }

    pub fn schedule(&self, voicemail: Voicemail) -> Result<()> {
        trace!("scheduling voicemail (id: {})", voicemail.id);

        self.queue.insert(
            voicemail.id,
            (voicemail.scheduled.context(GetScheduled)? - Utc::now().naive_utc())
                .to_std()
                .context(ConvertToStdDuration)?,
        );

        Ok(())
    }

    async fn show(&self, id: i32, context: Arc<BotContext>) -> Result<()> {
        trace!("showing voicemail (id: {})", id);

        let conn = &context.conn();

        let v = Voicemail::by_id(conn, id)
            .context(GetVoicemail)?
            .context(VoicemailNotFound { id })?;

        // disable voicemail
        v.set_active(conn, false).context(DisableVoicemail)?;

        let channel = Channel::by_id(conn, v.channel_id)
            .context(GetChannel)?
            .context(ChannelNotFound { id: v.channel_id })?;
        let channel_name = channel.name(conn).context(GetChannelName)?;

        let receiver = User::by_id(conn, v.receiver_id)
            .context(GetUser)?
            .context(UserNotFound { id: v.receiver_id })?;

        context
            .twitchbot()
            .writer()
            .privmsg(
                &channel_name,
                &format!(
                    "{}, one message for you: {}",
                    receiver.display_name_or_name(),
                    &v.to_string(conn)
                ),
            )
            .await
            .context(SendPrivmsg {
                channel: &channel_name,
            })
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
