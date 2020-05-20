#![feature(type_alias_impl_trait)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;
extern crate futures_delay_queue;
extern crate futures_executor;
extern crate humantime;
extern crate nom;
extern crate r2d2;
extern crate time;

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
