use config::Config;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use futures_executor::block_on;
use std::sync::Arc;
use twitchchat::Client;

type Pool = diesel::r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct Context {
    config: Config,
    pool: Pool,
    chat: Client,
}

impl Context {
    pub fn new(config: Config, pool: Pool) -> Arc<Self> {
        Arc::new(Self {
            config,
            pool,
            chat: Client::new(),
        })
    }

    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn chat(&self) -> &Client {
        &self.chat
    }

    pub fn join_channel(&self, channel: String) {
        let join = async {
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
        };
        block_on(join);
    }

    pub fn leave_channel(&self, channel: String) {
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
