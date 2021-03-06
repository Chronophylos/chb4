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
    twitchbot: TwitchBot,

    // voicemail scheduler
    scheduler: Arc<Scheduler>,

    // manpage index
    manpage_index: Arc<manpages::Index>,

    clock: Instant,

    pub version: &'static str,
    pub git_commit: &'static str,
}

impl BotContext {
    pub fn new(
        config: Config,
        pool: Pool,
        twitchbot: TwitchBot,
        manpage_index: manpages::Index,
    ) -> Arc<Self> {
        Arc::new(Self {
            config,
            pool,
            twitchbot,
            scheduler: Arc::new(Scheduler::new()),
            manpage_index: Arc::new(manpage_index),
            clock: Instant::now(),
            version: env!("CARGO_PKG_VERSION"),
            git_commit: env!("GIT_HASH"),
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
        self.config.get_str("twitch.name").unwrap()
    }

    pub fn twitchbot(&self) -> TwitchBot {
        self.twitchbot.clone()
    }

    pub fn scheduler(&self) -> Arc<Scheduler> {
        self.scheduler.clone()
    }

    pub async fn run_scheduler(this: Arc<Self>) {
        this.scheduler.run(this.clone()).await.unwrap()
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
