#![warn(clippy::result_unwrap_used)]
#![warn(clippy::option_unwrap_used)]
#![warn(clippy::dbg_macro)]
#![warn(clippy::todo)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

pub mod actions;
pub mod commands;
pub mod context;
pub mod database;
pub mod handler;
pub mod helpers;
pub mod manpages;
pub mod message;
pub mod models;
pub mod schema;
pub mod voicemail;

mod log_format;
mod stopwatch;
mod twitchbot;

pub use log_format::format;
pub use stopwatch::Stopwatch;
pub use twitchbot::TwitchBot;
