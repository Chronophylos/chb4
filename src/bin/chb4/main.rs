extern crate chrono;
extern crate config;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate bytes;
extern crate futures_util;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate lazy_static;

mod actions;
mod commands;

use chb4::context::Context;
use chb4::database;

use chrono::prelude::*;
use config::{Config, Environment, File, FileFormat};
use diesel::r2d2;
use diesel::PgConnection;
use std::env;
use twitchchat::{client::Error, client::Status, events, Client, Secure};
// so .next() can be used on the EventStream
// futures::stream::StreamExt will also work
use std::convert::TryInto;
use tokio::stream::StreamExt as _;

/// The main is currently full of bloat. The plan is to move everything into their own module
#[tokio::main]
async fn main() {
    flexi_logger::Logger::with_env_or_str("chb4=trace, debug")
        .format(chb4::format)
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    let version = env!("CARGO_PKG_VERSION");
    let git_hash = env!("GIT_HASH");

    info!("Starting CHB4 {} ({})", version, git_hash);

    let mut config = Config::new();
    config
        .merge(
            // look for config in system config directory
            File::with_name("/etc/chb4/config")
                .format(FileFormat::Toml)
                .required(false),
        )
        .unwrap_or_else(|e| panic!("Loading config from /etc/chb4 failed with {}", e))
        .merge(
            // look for config in working directory
            File::with_name("config")
                .format(FileFormat::Toml)
                .required(false),
        )
        .unwrap_or_else(|e| panic!("Loading config from current directory failed with {}", e))
        // look for config in environment
        .merge(Environment::with_prefix("CHB4").separator("_"))
        .unwrap_or_else(|e| panic!("Loading config from env failed with {}", e));
    info!("Loaded config");

    let manager =
        r2d2::ConnectionManager::<PgConnection>::new(config.get_str("database.url").unwrap());
    let pool = r2d2::Pool::builder().build(manager).unwrap();
    info!("Created Database Pool");

    let context = Context::new(config, pool);

    let client = context.chat();

    let actions = actions::handler::new(context.clone());
    info!("Created Action Handler");

    let commands = commands::handler::new(context.clone());

    info!("Created Command Handler");

    let nick = context.config().get_str("twitch.nick").unwrap();
    let pass = context.config().get_str("twitch.pass").unwrap();
    let channel = nick.clone();

    // connect via (tls or normal, 'Secure' determines that) tcp with this nick and password
    let (read, write) = twitchchat::connect_easy(&nick, &pass, Secure::UseTls)
        .await
        .unwrap();

    info!("Connected to {}", twitchchat::TWITCH_IRC_ADDRESS_TLS);

    // get a future that resolves when the client is done reading, fails to read/write or is
    // stopped
    let done = client.run(read, write);

    // subscribe to an event stream

    {
        // clone nick so we can use it here
        let nick = nick.clone();

        // for privmsg (what users send to channels)
        let mut privmsg = client.dispatcher().await.subscribe::<events::Privmsg>();

        // we can move the client to another task by cloning it
        let bot_client = client.clone();
        let context = context.clone();

        // spawn a task to consume the stream
        tokio::task::spawn(async move {
            while let Some(msg) = privmsg.next().await {
                trace!("Got PRIVMSG message");

                if msg.name == nick {
                    // message must be sent by the bot -> ignore it
                    trace!("dropping PRIVMSG since it was sent by the bot");
                    break;
                }

                {
                    let user_id = msg.user_id().unwrap().try_into().unwrap();
                    let name = msg.name.to_owned();
                    let display_name = msg.display_name().unwrap();
                    let now = Local::now();

                    let user = match database::user::bump(
                        &context.pool().get().unwrap(),
                        user_id,
                        &name,
                        &display_name,
                        &now,
                    ) {
                        Ok(u) => u,
                        Err(e) => {
                            error!("{}", e);
                            continue;
                        }
                    };

                    if user.banned(&now) {
                        trace!("User {} is banned. Ignoring message.", user.name);
                        continue;
                    }
                }

                {
                    let writer = bot_client.writer();

                    actions
                        .handle_privmsg(msg.clone(), &mut writer.clone())
                        .await;
                    commands
                        .handle_privmsg(msg.clone(), &mut writer.clone())
                        .await;
                }
            }
        });
    }

    // for join (when a user joins a channel)
    {
        let mut join = client.dispatcher().await.subscribe::<events::Join>();

        let join_client = client.clone();
        tokio::task::spawn(async move {
            while let Some(msg) = join.next().await {
                // we've joined a channel
                info!("Joined {}", msg.channel);

                if msg.channel == nick {
                    let mut writer = join_client.writer();

                    if let Err(err) = writer
                        .privmsg(&msg.channel, &format!("Connected with version {}", version))
                        .await
                    {
                        error!("Could not write to channel {}", err);
                    }
                }
            }
        });
    }

    {
        let mut handles = Vec::new();
        join_channel(client.clone(), channel).await;

        for channel in database::channel::all_enabled(&context.pool().get().unwrap()) {
            let handle = join_channel(client.clone(), channel);
            handles.push(handle);
        }

        for handle in handles.drain(..) {
            handle.await;
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

async fn join_channel(client: Client, channel: String) {
    info!("Joining channel {}", &channel);

    // get a clonable writer from the client
    // join a channel, methods on writer return false if the client is disconnected
    if let Err(err) = client.writer().join(&channel).await {
        match err {
            Error::InvalidChannel(..) => {
                error!("could not join channel because the name is empty");
            }
            _ => {
                error!("got an error, but I don't know what to do: {}", err);
                // we'll get an error if we try to write to a disconnected client.
                // if this happens, you should shutdown your tasks
            }
        }
    }
}