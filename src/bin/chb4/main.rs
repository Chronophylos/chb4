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
    database::{self, Channel, Voicemail},
    handler::Twitch,
    manpages, TwitchBot,
};

use config::{Config, Environment, File, FileFormat};
use diesel::r2d2::{ConnectionManager, Pool};
use snafu::{ResultExt, Snafu};
use std::{env, sync::Arc};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Initializing logger: {}", source))]
    InitLogger {
        source: flexi_logger::FlexiLoggerError,
    },

    #[snafu(display("Loading config from {}: {}", target, source))]
    LoadConfig {
        target: &'static str,
        source: config::ConfigError,
    },

    #[snafu(display("Loading config entry: {}", source))]
    GetConfigEntry { source: config::ConfigError },

    #[snafu(display("Building R2D2 Pool: {}", source))]
    BuildR2D2Pool { source: r2d2::Error },

    #[snafu(display("Getting enabled channels: {}", source))]
    GetEnabledChannels { source: database::channel::Error },
}

/// The main is currently full of bloat. The plan is to move everything into their own module
#[tokio::main]
async fn main() -> Result<(), Box<Error>> {
    flexi_logger::Logger::with_env_or_str("chb4=trace, rustls=info, debug")
        .format(chb4::format)
        .start()
        .context(InitLogger)?;

    let version = env!("CARGO_PKG_VERSION");
    let git_hash = env!("GIT_HASH");

    info!("Starting CHB4 {} ({})", version, git_hash);

    let mut config = Config::new();
    config
        // look for config in system config directory
        .merge(
            File::with_name("/etc/chb4/config")
                .format(FileFormat::Toml)
                .required(false),
        )
        .context(LoadConfig {
            target: "/etc/chb4",
        })?
        // look for config in working directory
        .merge(
            File::with_name("config")
                .format(FileFormat::Toml)
                .required(false),
        )
        .context(LoadConfig {
            target: "current directory",
        })?
        // look for config in environment
        .merge(Environment::with_prefix("CHB4").separator("_"))
        .context(LoadConfig {
            target: "environment",
        })?;

    info!("Loaded config");

    let manager = ConnectionManager::new(config.get_str("database.url").context(GetConfigEntry)?);
    let pool = Pool::builder().build(manager).context(BuildR2D2Pool)?;
    debug!("Created Database Pool");

    let (twitchbot, runner) = TwitchBot::new();

    let mut context = BotContext::new(config, pool, twitchbot);
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

    let twitch_handlers = [
        Arc::new(action_handler) as Arc<dyn Twitch>,
        Arc::new(command_handler) as Arc<dyn Twitch>,
    ];

    let channels = {
        let bot_channel = context.bot_name();
        let conn = &context.conn();

        // ensure the bot channel is in the database
        let _ = Channel::join(conn, &bot_channel); // ignore result

        Channel::all_enabled(conn).context(GetEnabledChannels)?
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

    let name = context.config().get_str("twitch.name").unwrap();
    let token = context.config().get_str("twitch.token").unwrap();

    let twitchbot = context.twitchbot();
    // await for the client to be done
    debug!("Waiting for futures to resolve");
    let (scheduler_result, twitchbot_result) = tokio::join!(
        // scheduler
        BotContext::run_scheduler(context.clone()),
        // twitchbot
        twitchbot.start(
            runner,
            context.clone(),
            name,
            token,
            Arc::new(twitch_handlers),
            channels
        ),
    );

    debug!("Futures resolved {:?}", twitchbot_result);
    debug!("Futures resolved {:?}", scheduler_result);

    Ok(())
}
