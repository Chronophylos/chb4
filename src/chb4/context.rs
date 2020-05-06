use crate::{handler::Twitch, manpages, twitchbot, voicemail::Scheduler, TwitchBot};
use config::Config;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use std::{
    sync::{Arc, RwLock},
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
    twitchbot: Arc<RwLock<TwitchBot>>,

    // voicemail scheduler
    scheduler: Arc<Scheduler>,

    // manpage index
    manpage_index: Arc<manpages::Index>,

    clock: Instant,
    pub version: &'static str,
}

impl BotContext {
    pub fn new(config: Config, pool: Pool) -> Arc<Self> {
        Arc::new(Self {
            config,
            pool,
            twitchbot: Arc::new(RwLock::new(TwitchBot::new())),
            scheduler: Arc::new(Scheduler::new()),
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
        self.config.get_str("twitch.name").unwrap()
    }

    pub fn twitchbot(&self) -> Arc<RwLock<TwitchBot>> {
        self.twitchbot.clone()
    }

    pub fn scheduler(&self) -> Arc<Scheduler> {
        self.scheduler.clone()
    }

    pub async fn run_scheduler(this: Arc<Self>) {
        this.scheduler.run(this.clone()).await.unwrap()
    }

    pub async fn connect_twitchbot(
        this: Arc<Self>,
        handlers: &[Arc<dyn Twitch>],
        initial_channels: Vec<String>,
    ) -> Result<(), twitchbot::Error> {
        let name = this.config.get_str("twitch.name").unwrap();
        let token = this.config.get_str("twitch.token").unwrap();

        this.twitchbot()
            .write()
            .unwrap()
            .start(this, name, token, handlers, initial_channels)
            .await
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
