use crate::{context::BotContext, database::User, handler::Twitch, Stopwatch};
use chrono::prelude::*;
use futures_executor::block_on;
use snafu::{ResultExt, Snafu};
use std::{convert::TryInto, sync::Arc};
use tokio::stream::StreamExt as _;
use twitchchat::{events, messages, Control, Dispatcher, RateLimit, Runner, Status, Writer};

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

#[derive(Clone, Default)]
pub struct TwitchBot {
    control: Option<Control>,
    writer: Option<Writer>,
    name: Option<String>,
}

impl TwitchBot {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn start(
        &mut self,
        context: Arc<BotContext>,
        name: String,
        token: String,
        handlers: Vec<Arc<dyn Twitch>>,
        initial_channels: Vec<String>,
    ) -> Result<()> {
        self.name = Some(name.clone());

        let dispatcher = Dispatcher::new();
        let (runner, mut control) = Runner::new(dispatcher.clone(), RateLimit::default());
        self.control = Some(control.clone());
        self.writer = Some(control.writer().to_owned());

        let bot = self.run(context, dispatcher.clone(), handlers, initial_channels);

        let stream = twitchchat::connect_easy_tls(&name, &token).await.unwrap();
        let done = runner.run(stream);

        tokio::select! {
            _ = bot => { warn!("bot stopped") }
            status = done => {
                match status {
                    Ok(Status::Canceled) => { warn!("runner was canceled") }
                    Ok(Status::Eof) => { info!("Got an EOF, exiting") }
                    Ok(Status::Timeout) => { error!("Client connection timed out") }
                    Err(err) => {
                        eprintln!("Error running TwitchChat: {}", err);
                        return Err(Error::RunRunner { source: err });
                    }
                }
            }
        }

        Ok(())
    }

    async fn run(
        &mut self,
        context: Arc<BotContext>,
        dispatcher: Dispatcher,
        handlers: Vec<Arc<dyn Twitch>>,
        channels: Vec<String>,
    ) {
        // get a writer clone
        let mut writer = self.writer.clone().unwrap();

        // subscribe to the events we're interested in
        let mut privmsg = dispatcher.subscribe::<events::Privmsg>();
        let mut join = dispatcher.subscribe::<events::Join>();

        // and wait for a specific event (blocks the current task)
        let ready = dispatcher.wait_for::<events::IrcReady>().await.unwrap();
        info!(
            "Connected to {} as {}",
            twitchchat::TWITCH_IRC_ADDRESS_TLS,
            ready.nickname
        );

        // and then join some channels
        info!("Joining twitch channels (count: {})", channels.len());
        for channel in channels {
            debug!("Joining channel (name: {})", channel);
            writer.join(channel).await.unwrap();
        }

        // and then our 'main loop'
        // todo: offload handling to other threads
        loop {
            tokio::select! {
                Some(msg) = privmsg.next() => {
                    trace!("Got chat message (provider: twitch, channel: {})", &msg.channel);
                    self.handle_privmsg(context.clone(), &handlers, &msg).await;
                },
                Some(msg) = join.next() => {
                    trace!("Got join message (provider: twitch, channel: {})", &msg.channel);
                    self.handle_join(context.clone(), &msg).await;
                },
                else => break,
            }
        }
    }

    async fn handle_privmsg(
        &self,
        context: Arc<BotContext>,
        handlers: &Vec<Arc<dyn Twitch>>,
        msg: &messages::Privmsg<'_>,
    ) {
        // this variable name should not be changed.
        // having no name or _  as name just drops the Stopwatch instantly.
        // and having no _ infront annoys the compiler
        let _stopwatch = Stopwatch::new(|d| {
            debug!("Handling PRIVMSG took: {}", d);
        });

        trace!("Got PRIVMSG message");

        if msg.name == self.name.clone().unwrap() {
            // message must be sent by the bot -> ignore it
            trace!("ignoring PRIVMSG since it was sent by the bot");
            return;
        }

        // bump the user in database
        let user = {
            // todo: check all of the unwraps for errors
            let user_id = msg.user_id().unwrap().try_into().unwrap();
            let name = msg.name.to_owned();
            let display_name = msg.display_name().unwrap();
            let now = Local::now();

            let user = match User::bump(&context.conn(), user_id, &name, &display_name, &now) {
                Ok(u) => u,
                Err(e) => {
                    error!("{}", e);
                    return;
                }
            };

            if user.banned(&now) {
                trace!("User {} is banned. Ignoring message.", user.name);
                return;
            }

            user
        };

        for handler in handlers {
            handler.handle(Arc::new(msg.clone()), &user).await;
        }
    }

    async fn handle_join(&self, context: Arc<BotContext>, msg: &messages::Join<'_>) {
        if msg.channel == self.name.clone().unwrap() {
            // we've joined a channel
            info!("Joined {}", msg.channel);

            if let Err(err) = self
                .writer
                .clone()
                .unwrap()
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

    pub fn stop(&self) {
        let control = self.control.clone().unwrap();
        tokio::spawn(async move { control.stop() });
    }

    pub fn join(&self, channel: &str) -> Result<()> {
        block_on(async { self.writer.clone().unwrap().join(channel).await })
            .context(JoinChannel { channel })
    }

    pub fn part(&self, channel: &str) -> Result<()> {
        block_on(async { self.writer.clone().unwrap().part(channel).await })
            .context(PartChannel { channel })
    }

    pub fn writer(self) -> Writer {
        self.writer.unwrap().clone()
    }
}
