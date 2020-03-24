extern crate chrono;
extern crate config;
extern crate evalexpr;
extern crate humantime;
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

use chb4::{
    actions::ActionHandler,
    commands::CommandHandler,
    context::BotContext,
    database::{Channel, User, Voicemail},
    handler::Handler,
    Stopwatch,
};
use chrono::prelude::*;
use config::{Config, Environment, File, FileFormat};
use diesel::{r2d2, PgConnection};
use std::{convert::TryInto, env};
use tokio::stream::StreamExt as _;
use twitchchat::{client::Status, events, Secure}; // so .next() can be used on the EventStream

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
    debug!("Created Database Pool");

    let context = BotContext::new(config, pool);
    debug!("Created Bot Context");

    let action_index = actions::all(context.clone());
    let command_index = commands::all(context.clone());

    let action_handler = ActionHandler::new(context.clone(), action_index);
    debug!("Created Action Handler");

    let command_handler = CommandHandler::new(context.clone(), command_index);
    debug!("Created Command Handler");

    let handlers: Vec<Box<dyn Handler>> = vec![Box::new(action_handler), Box::new(command_handler)];

    let client = context.chat();

    // get nick and password from config
    let nick = context.bot_name();
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

    // Subscribe to privmsg event stream.
    // Here everything happens. Like checking for commands, actions and bumping the user in the
    // database
    {
        // clone nick so we can use it here
        let nick = nick.clone();

        // for privmsg (what users send to channels)
        let mut privmsg = client.dispatcher().await.subscribe::<events::Privmsg>();

        let context = context.clone();

        // spawn a task to consume the stream
        tokio::task::spawn(async move {
            while let Some(msg) = privmsg.next().await {
                // this variable name should not be changed.
                // having no name or _  as name just drops the Stopwatch instantly.
                // and having no _ infront annoys the compiler
                let _stopwatch = Stopwatch::new(|d| {
                    debug!("Handling PRIVMSG took: {}", d);
                });

                trace!("Got PRIVMSG message");

                if msg.name == nick {
                    // message must be sent by the bot -> ignore it
                    trace!("ignoring PRIVMSG since it was sent by the bot");
                    continue;
                }

                // bump the user in database
                let user = {
                    // todo check all of the unwraps for errors
                    let user_id = msg.user_id().unwrap().try_into().unwrap();
                    let name = msg.name.to_owned();
                    let display_name = msg.display_name().unwrap();
                    let now = Local::now();

                    let user =
                        match User::bump(&context.conn(), user_id, &name, &display_name, &now) {
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

                    user
                };

                {
                    for handler in &handlers {
                        handler.handle(msg.clone(), &user).await;
                    }
                }
            }
        });
    }

    // Subscribe to join event stream.
    // BUG: For some reason this spams Joined when joining channels from the database
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

    // Join channels. First join the bots channel, then get all enabled channels from the database
    // and join them.
    {
        let conn = &context.conn();
        // ensure the bot channel is in the database
        let _ = Channel::join(conn, &channel); // ignore result
        context.join_channel(channel).await;

        for channel in Channel::all_enabled(conn).unwrap() {
            context.join_channel(channel).await;
        }
    }

    // schedule voicemails
    {
        let context = context.clone();
        tokio::task::spawn(async move {
            let conn = &context.conn();
            let voicemails = match Voicemail::active_scheduled(conn) {
                Ok(v) => match v {
                    Some(v) => v,
                    None => {
                        info!("no scheduled voicemails found");
                        return;
                    }
                },
                Err(e) => {
                    error!("Could not get voicemails for scheduling: {}", e);
                    return;
                }
            };

            for voicemail in voicemails {
                context.scheduler().schedule(voicemail);
            }
        });
    }

    // await for the client to be done
    let (tmi_result, _) = tokio::join!(
        // await tmi client
        done,
        // await scheduler
        context.scheduler().run(context.clone()),
    );

    match tmi_result {
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
