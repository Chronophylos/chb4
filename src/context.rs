use config::Config;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::MysqlConnection;

type ThePool = Pool<ConnectionManager<MysqlConnection>>;

#[derive(Clone)]
pub struct Context {
    config: Config,
    pool: ThePool,
}

impl Context {
    pub fn new(config: Config, pool: ThePool) -> Self {
        Self { config, pool }
    }

    pub fn pool(&self) -> &ThePool {
        &self.pool
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}
