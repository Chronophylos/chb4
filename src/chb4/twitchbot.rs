use crate::{context::BotContext, database::User, handler::Twitch, Stopwatch};
use chrono::prelude::*;
use futures_executor::block_on;
use snafu::{ResultExt, Snafu};
use std::{convert::TryInto, sync::Arc};
use tokio::stream::StreamExt as _;
use twitchchat::{events, Control, Dispatcher, IntoChannel, Runner, Status, Writer};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Connection to {}: {}", server, source))]
    Connect {
        server: &'static str,
        source: std::io::Error,
    },

    #[snafu(display("Waiting for IRC Ready message: {}", source))]
    WaitForIrcReady { source: twitchchat::Error },

    #[snafu(display("Getting twitch username from config: {}", source))]
    GetNameFromConfig { source: config::ConfigError },

    #[snafu(display("Getting twitch token from config: {}", source))]
    GetTokenFromConfig { source: config::ConfigError },

    #[snafu(display("Running runner: {}", source))]
    RunRunner { source: twitchchat::Error },

    #[snafu(display("Reconnecting to server: {}", source))]
    Reconnect { source: twitchchat::Error },

    #[snafu(display("Joining channel (name: {}): {}", channel, source))]
    JoinChannel {
        source: twitchchat::Error,
        channel: String,
    },

    #[snafu(display("Parting channel (name: {}): {}", channel, source))]
    PartChannel {
        source: twitchchat::Error,
        channel: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct TwitchBot {
    writer: Writer,
    control: Control,
    dispatcher: Dispatcher,
}

impl TwitchBot {
    pub fn new(mut control: Control, dispatcher: Dispatcher) -> Arc<Self> {
        Arc::new(Self {
            writer: control.writer().clone(),
            control,
            dispatcher,
        })
    }

    pub async fn run<C>(
        self,
        context: Arc<BotContext>,
        runner: Runner,
        handlers: Vec<Arc<dyn Twitch>>,
        channels: Vec<C>,
    ) -> Result<Status>
    where
        C: IntoChannel,
    {
        let config = context.config();
        let name = config.get_str("twitch.name").context(GetNameFromConfig)?;
        let token = config.get_str("twitch.pass").context(GetTokenFromConfig)?;
        let mut writer = self.writer.clone();

        // create connection
        let conn = twitchchat::connect_easy_tls(&name, &token)
            .await
            .context(Connect {
                server: twitchchat::TWITCH_IRC_ADDRESS_TLS,
            })?;

        // subscribe to events
        let mut privmsg = self.dispatcher.subscribe::<events::Privmsg>();
        let mut join = self.dispatcher.subscribe::<events::Join>();

        // wait until irc is ready
        let ready = self
            .dispatcher
            .wait_for::<events::IrcReady>()
            .await
            .context(WaitForIrcReady)?;
        info!(
            "Connected to {} as {}",
            twitchchat::TWITCH_IRC_ADDRESS_TLS,
            ready.nickname
        );

        // join channels
        for channel in channels {
            match writer.join(channel).await {
                Ok(_) => {}
                Err(e) => error!("Could not send join message: {}", e),
            }
        }

        // start loops
        {
            let context = context.clone();
            tokio::task::spawn(async move {
                while let Some(msg) = privmsg.next().await {
                    // this variable name should not be changed.
                    // having no name or _  as name just drops the Stopwatch instantly.
                    // and having no _ infront annoys the compiler
                    let _stopwatch = Stopwatch::new(|d| {
                        debug!("Handling PRIVMSG took: {}", d);
                    });

                    trace!("Got PRIVMSG message");

                    if msg.name == name {
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

                        let user = match User::bump(
                            &context.conn(),
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

                        user
                    };

                    for handler in &handlers {
                        handler.handle(msg.clone(), &user).await;
                    }
                }
            });
        }

        {
            let context = context.clone();
            tokio::task::spawn(async move {
                let bot_name = context.bot_name();
                while let Some(msg) = join.next().await {
                    // we've joined a channel
                    info!("Joined {}", msg.channel);

                    if msg.channel == bot_name {
                        if let Err(err) = writer
                            .privmsg(
                                &msg.channel,
                                &format!("Connected with version {}", context.version),
                            )
                            .await
                        {
                            error!("Could not write to channel {}", err);
                        }
                    }
                }
            });
        }

        // run dispatcher/writer loop
        Ok(runner.run(conn).await.context(RunRunner)?)
    }

    pub fn writer(&self) -> Writer {
        self.writer.clone()
    }

    pub fn stop(&self) {
        self.control.stop();
    }

    pub fn reconnect(&mut self) -> Result<()> {
        block_on(async { self.writer.disconnect().await }).context(Reconnect)
    }

    pub fn join_sync(&self, channel: &str) -> Result<()> {
        let mut writer = self.writer.clone();
        block_on(async { writer.join(channel).await }).context(JoinChannel {
            channel: channel.to_string(),
        })
    }

    pub fn part_sync(&self, channel: &str) -> Result<()> {
        let mut writer = self.writer.clone();
        block_on(async { writer.part(channel).await }).context(PartChannel {
            channel: channel.to_string(),
        })
    }
}

impl Drop for TwitchBot {
    fn drop(&mut self) {
        self.dispatcher.clear_subscriptions_all();
    }
}
