use crate::context::BotContext;
use crate::database::{Channel, User, Voicemail};
use chrono::prelude::*;
use futures_delay_queue::{delay_queue, DelayQueue, Receiver};
use std::sync::Arc;

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

    pub async fn run(&self, context: Arc<BotContext>) {
        debug!("starting scheduler loop");
        loop {
            if let Some(id) = self.receiver.receive().await {
                self.show(id, context.clone()).await
            }
        }
    }

    pub fn schedule(&self, voicemail: Voicemail) {
        trace!("scheduling voicemail (id: {})", voicemail.id);

        self.queue.insert(
            voicemail.id,
            (voicemail.scheduled.unwrap() - Utc::now().naive_utc())
                .to_std()
                .unwrap(),
        );
    }

    async fn show(&self, id: i32, context: Arc<BotContext>) {
        trace!("showing voicemail (id: {})", id);

        let conn = &context.conn();

        let v = Voicemail::by_id(conn, id).unwrap().unwrap();

        match v.set_active(conn, false) {
            Ok(_) => {}
            Err(e) => {
                error!("Could not disable voicemail (id: {}): {}", id, e);
                return;
            }
        };

        let channel = Channel::by_id(conn, v.channel_id).unwrap().unwrap();
        let channel_name = channel.name(conn).unwrap();

        let receiver = User::by_id(conn, v.receiver_id).unwrap().unwrap();

        context
            .twitchbot()
            .read()
            .unwrap()
            .clone()
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
            .unwrap();
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
