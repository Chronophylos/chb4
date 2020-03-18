use crate::context::Context;
use crate::database::{Channel, Voicemail};
use chrono::prelude::*;
use futures_delay_queue::{delay_queue, DelayQueue, Receiver};

pub struct Scheduler {
    queue: DelayQueue<i32>,
    receiver: Receiver<i32>,
    context: Context,
}

impl Scheduler {
    pub fn new(context: Context) -> Self {
        // create a queue with maximum size
        let (queue, receiver) = delay_queue(std::usize::MAX);
        Self {
            queue,
            receiver,
            context,
        }
    }

    pub async fn run(&self) {
        loop {
            if let Some(id) = self.receiver.receive().await {
                self.show(id).await
            }
        }
    }

    pub fn schedule(&self, voicemail: Voicemail) {
        self.queue.insert(
            voicemail.id,
            (voicemail.scheduled.unwrap() - Utc::now().naive_utc())
                .to_std()
                .unwrap(),
        );
    }

    async fn show(&self, id: i32) {
        let conn = &self.context.conn();
        let v = Voicemail::by_id(conn, id).unwrap().unwrap();
        let channel = Channel::by_id(conn, v.channel_id).unwrap().unwrap();
        let channel_name = channel.name(conn).unwrap();
        self.context
            .chat()
            .writer()
            .privmsg(channel_name, &v.to_string(conn))
            .await
            .unwrap();
    }
}
