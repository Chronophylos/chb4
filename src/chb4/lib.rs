#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;
extern crate r2d2;

pub mod context;
pub mod database;
pub mod helpers;
pub mod models;
pub mod schema;

mod documentation;
mod log_format;

pub use documentation::Documentation;
pub use log_format::format;
