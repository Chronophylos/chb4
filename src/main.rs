#![warn(clippy::all)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate diesel;
extern crate chrono;
extern crate config;
extern crate regex;

mod actions;
mod commands;
mod db;
mod log_format;
mod models;
mod schema;

use chrono::prelude::*;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use twitchchat::{client::Error, client::Status, events, Client, Secure};
// so .next() can be used on the EventStream
// futures::stream::StreamExt will also work
use std::env;
use tokio::stream::StreamExt as _;

/// The main is currently full of bloat. The plan is to move everything into their own module
#[tokio::main]
async fn main() {
    // should only be run once
    flexi_logger::Logger::with_env_or_str("chb4=trace, debug")
        .format(log_format::format)
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    let version = env!("CARGO_PKG_VERSION");
    let git_hash = env!("GIT_HASH");

    info!("Starting CHB4 {} ({})", version, git_hash);

    let mut settings = config::Config::new();
    settings
        .merge(
            // look for config in system config directory
            config::File::with_name("/etc/chb4/config")
                .format(config::FileFormat::Toml)
                .required(false),
        )
        .unwrap_or_else(|e| panic!("Loading config from /etc/chb4 failed with {}", e))
        .merge(
            // look for config in working directory
            config::File::with_name("config")
                .format(config::FileFormat::Toml)
                .required(false),
        )
        .unwrap_or_else(|e| panic!("Loading config from current directory failed with {}", e))
        // look for config in environment
        .merge(config::Environment::with_prefix("CHB4").separator("_"))
        .unwrap_or_else(|e| panic!("Loading config from env failed with {}", e));
    info!("Loaded config");

    let actions = actions::handler::new();
    info!("Created Action Handler");

    let commands = commands::handler::new();

    info!("Created Command Handler");

    let db = connect_to_db(&settings);
    info!("Connected to Database");

    let nick = settings.get_str("twitch.nick").unwrap();
    let pass = settings.get_str("twitch.pass").unwrap();
    let channel = nick.clone();

    // connect via (tls or normal, 'Secure' determines that) tcp with this nick and password
    let (read, write) = twitchchat::connect_easy(&nick, &pass, Secure::UseTls)
        .await
        .unwrap();

    info!("Connected to {}", twitchchat::TWITCH_IRC_ADDRESS_TLS);

    // make a client. the client is clonable
    let client = Client::new();

    // get a future that resolves when the client is done reading, fails to read/write or is
    // stopped
    let done = client.run(read, write);

    // subscribe to an event stream

    {
        // for privmsg (what users send to channels)
        let mut privmsg = client.dispatcher().await.subscribe::<events::Privmsg>();

        // spawn a task to consume the stream
        tokio::task::spawn(async move {
            while let Some(msg) = privmsg.next().await {
                trace!("Got PRIVMSG message");

                let tags: &twitchchat::Tags = &msg.tags;
                let user_id: String = tags.get_parsed("user-id").unwrap();
                let name = msg.name.to_owned();
                let display_name: String = tags.get_parsed("display-name").unwrap();

                db::bump_user(
                    &db,
                    &user_id,
                    &name,
                    &display_name,
                    &Local::now().naive_local(),
                );
            }
        });
    }

    // for join (when a user joins a channel)
    {
        let mut join = client.dispatcher().await.subscribe::<events::Join>();

        let join_client = client.clone();
        tokio::task::spawn(async move {
            while let Some(msg) = join.next().await {
                let mut writer = join_client.writer();
                trace!("Got JOIN message");
                // we've joined a channel
                if msg.name == nick {
                    info!("Joined {}", msg.channel);
                    if let Err(err) = writer
                        .privmsg(&msg.channel, &format!("Connected with version {}", version))
                        .await
                    {
                        error!("Could not write to channel {}", err);
                        break;
                    }
                    break; // returning/dropping the stream un-subscribes it
                }
            }
        });
    }

    // for privmsg again
    {
        let mut bot = client.dispatcher().await.subscribe::<events::Privmsg>();

        // we can move the client to another task by cloning it
        let bot_client = client.clone();
        tokio::task::spawn(async move {
            // get writer from cloned client so we dont move the original
            let writer = bot_client.writer();
            while let Some(msg) = bot.next().await {
                actions.handle_privmsg(&msg, &mut writer.clone());
                commands.handle_privmsg(&msg, &mut writer.clone()).await;
            }
        });
    }

    info!("Joining channel {}", &channel);
    // get a clonable writer from the client
    // join a channel, methods on writer return false if the client is disconnected
    if let Err(err) = client.writer().join(&channel).await {
        match err {
            Error::InvalidChannel(..) => {
                error!("could not join channel because the name is empty");
                std::process::exit(1);
            }
            _ => {
                error!("got an error, but I don't know what to do: {}", err);
                // we'll get an error if we try to write to a disconnected client.
                // if this happens, you should shutdown your tasks
            }
        }
    }

    // await for the client to be done
    match done.await {
        Ok(Status::Eof) => {
            info!("done!");
        }
        Ok(Status::Canceled) => {
            info!("client was stopped by user");
        }
        Err(err) => {
            error!("error: {}", err);
        }
    }

    // note you should wait for all of your tasks to join before exiting
    // but we detached them to make this shorter

    debug!("clearing twitchchat subscriptions");
    // another way would be to clear all subscriptions
    // clearing the subscriptions would close each event stream
    client.dispatcher().await.clear_subscriptions_all();
}

fn connect_to_db(s: &config::Config) -> MysqlConnection {
    let database_url = s.get_str("database.url").unwrap();

    MysqlConnection::establish(&database_url).unwrap_or_else(|e| {
        panic!(
            "Connecting to database at {} failed with: {}",
            database_url, e
        )
    })
}
