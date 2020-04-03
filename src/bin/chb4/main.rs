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
    database::{Channel, Voicemail},
    handler::Twitch,
    manpages, TwitchBot,
};

use config::{Config, Environment, File, FileFormat};
use diesel::{r2d2, PgConnection};
use std::{env, sync::Arc};
use twitchchat::{Dispatcher, RateLimit, Runner, Status};

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

    let dispatcher = Dispatcher::new();
    let (runner, control) = Runner::new(dispatcher.clone(), RateLimit::default());
    let twitchbot = TwitchBot::new(control, dispatcher.clone());

    let manager =
        r2d2::ConnectionManager::<PgConnection>::new(config.get_str("database.url").unwrap());
    let pool = r2d2::Pool::builder().build(manager).unwrap();
    debug!("Created Database Pool");

    let mut context = BotContext::new(config, pool, twitchbot.clone());
    debug!("Created Bot Context");

    let action_index = actions::all(context.clone());
    let command_index = commands::all(context.clone());

    let mut manpage_index = manpages::Index::new();
    manpage_index.populate(action_index.clone());
    manpage_index.populate(command_index.clone());
    debug!("Created and populated Manpages");

    Arc::make_mut(&mut context).set_manpage_index(Arc::new(manpage_index));

    let action_handler = ActionHandler::new(context.clone(), action_index);
    debug!("Created Action Handler");

    let command_handler = CommandHandler::new(context.clone(), command_index);
    debug!("Created Command Handler");

    let twitch_handlers: Vec<Arc<dyn Twitch>> = vec![
        Arc::new(action_handler) as Arc<dyn Twitch>,
        Arc::new(command_handler) as Arc<dyn Twitch>,
    ];

    let channels = {
        let bot_channel = context.bot_name();
        let conn = &context.conn();

        // ensure the bot channel is in the database
        let _ = Channel::join(conn, &bot_channel); // ignore result

        Channel::all_enabled(conn).unwrap()
    };

    debug!("Found {} channels to join", channels.len());

    // schedule voicemails
    {
        let context = context.clone();
        tokio::task::spawn(async move {
            trace!("Scheduling old voicemails");
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

    {
        let twitchbot = twitchbot.clone();
        let context = context.clone();
        tokio::task::spawn(async move {
            if let Err(e) = twitchbot.start(context, twitch_handlers, channels).await {
                error!("Could not start TwitchBot: {}", e);
            }
        });
    }

    // await for the client to be done
    debug!("Waiting for futures");
    tokio::select!(
        // twitchbot
        status = twitchbot.connect(runner, context.clone()) => {
            match status {
                Ok(Status::Eof) => {
                    info!("TwitchBot got a EOF");
                }
                Ok(Status::Canceled) => {
                    info!("TwitchBot was stopped by user");
                }
                Err(e) => {
                    error!("Error connecting TwitchBot: {}", e);
                }
            }
        },
        // scheduler
        _ = context.scheduler().run(context.clone()) => {
            info!("Finished running Voicemail Scheduler")
        },
    );

    twitchbot.cleanup();
}
