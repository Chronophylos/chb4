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
    manpages,
};

use config::{Config, Environment, File, FileFormat};
use diesel::{r2d2, PgConnection};
use std::{env, sync::Arc};

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

    let mut context = BotContext::new(config, pool);
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
                context.scheduler().schedule(voicemail).unwrap();
            }
        });
    }

    // await for the client to be done
    debug!("Waiting for futures to resolve");
    let (twitchbot_result, _) = tokio::join!(
        // twitchbot
        BotContext::connect_twitchbot(context.clone(), twitch_handlers, channels),
        // scheduler
        BotContext::run_scheduler(context.clone()),
    );

    debug!("Features resolved {:?}", twitchbot_result);

    //match twitchbot_result {
    //    Ok(Status::Eof) => {
    //        info!("TwitchBot got a EOF");
    //    }
    //    Ok(Status::Canceled) => {
    //        info!("TwitchBot was stopped by user");
    //    }
    //    Err(e) => {
    //        error!("Error connecting TwitchBot: {}", e);
    //    }
    //}
}
