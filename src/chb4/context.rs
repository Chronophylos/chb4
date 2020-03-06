use config::Config;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;

type Pool = diesel::r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct Context {
    config: Config,
    pool: Pool,
}

impl Context {
    pub fn new(config: Config, pool: Pool) -> Self {
        Self { config, pool }
    }

    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}
