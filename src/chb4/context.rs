use crate::voicemail::Scheduler;
use config::Config;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use futures_executor::block_on;
use std::sync::Arc;
use twitchchat::Client;

pub type Connection = crate::database::Connection;

type Pool = diesel::r2d2::Pool<ConnectionManager<Connection>>;
type Conn = PooledConnection<ConnectionManager<Connection>>;

#[derive(Clone)]
pub struct Context {
    config: Config,
    pool: Pool,
    chat: Client,
    scheduler: Scheduler,
}

impl Context {
    pub fn new(config: Config, pool: Pool) -> Arc<Self> {
        Arc::new(Self {
            config,
            pool,
            chat: Client::new(),
            scheduler: Scheduler::new(),
        })
    }

    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    pub fn conn(&self) -> Conn {
        self.pool.get().unwrap()
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn bot_name(&self) -> String {
        self.config.get_str("twitch.nick").unwrap()
    }

    pub fn chat(&self) -> &Client {
        &self.chat
    }

    pub fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }

    /// Join channel blocking.
    pub fn join_channel_sync(&self, channel: String) {
        block_on(self.join_channel(channel));
    }

    /// Join channel non-blocking.
    pub async fn join_channel(&self, channel: String) {
        info!("Joining channel {}", channel);

        if let Err(err) = self.chat.writer().join(&channel).await {
            match err {
                twitchchat::client::Error::InvalidChannel(..) => {
                    error!("could not join channel because the name is empty");
                }
                _ => {
                    error!("got an error, but I don't know what to do: {}", err);
                }
            }
        }
    }

    /// Leave channel blocking.
    pub fn leave_channel_sync(&self, channel: String) {
        info!("Leaving channel {}", channel);

        let leave = async {
            if let Err(err) = self.chat.writer().part(&channel).await {
                match err {
                    twitchchat::client::Error::InvalidChannel(..) => {
                        error!("could not leave channel because the name is empty");
                    }
                    _ => {
                        error!("got an error, but I don't know what to do: {}", err);
                    }
                }
            }
        };

        block_on(leave);
    }
}
