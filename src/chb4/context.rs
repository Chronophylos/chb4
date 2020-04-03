use crate::{manpages, voicemail::Scheduler, TwitchBot};
use config::Config;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

pub type Connection = crate::database::Connection;

type Manager = ConnectionManager<Connection>;
type Pool = diesel::r2d2::Pool<Manager>;
type Conn = PooledConnection<Manager>;

#[derive(Clone)]
pub struct BotContext {
    config: Config,
    // connection pool for database
    pool: Pool,

    // bot for twitch
    twitchbot: Arc<TwitchBot>,

    // voicemail scheduler
    scheduler: Scheduler,

    // manpage index
    manpage_index: Arc<manpages::Index>,

    clock: Instant,
    pub version: &'static str,
}

impl BotContext {
    pub fn new(config: Config, pool: Pool, twitchbot: Arc<TwitchBot>) -> Arc<Self> {
        Arc::new(Self {
            config,
            pool,
            twitchbot,
            scheduler: Scheduler::new(),
            manpage_index: Arc::new(manpages::Index::new()),
            clock: Instant::now(),
            version: env!("CARGO_PKG_VERSION"),
        })
    }

    pub fn set_manpage_index(&mut self, index: Arc<manpages::Index>) {
        self.manpage_index = index;
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

    pub fn twitchbot(&self) -> Arc<TwitchBot> {
        self.twitchbot.clone()
    }

    pub fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }

    pub fn whatis(
        &self,
        chapter: Option<manpages::ChapterName>,
        name: String,
    ) -> Option<Arc<manpages::Manpage>> {
        self.manpage_index.whatis(chapter, name)
    }

    /// Get the duration how long ago this context was created
    pub fn elapsed(&self) -> Duration {
        self.clock.elapsed()
    }
}
