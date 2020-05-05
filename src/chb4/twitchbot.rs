use crate::{
    context::BotContext,
    database::{self, User},
    handler::Twitch,
    Stopwatch,
};
use chrono::prelude::*;
use futures_executor::block_on;
use snafu::{OptionExt, ResultExt, Snafu};
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

    #[snafu(display("Sending privmsg (channel: {}): {}", channel, source))]
    SendPrivmsg {
        source: twitchchat::Error,
        channel: String,
    },

    #[snafu(display("Bumping user: {}", source))]
    BumpUser { source: database::user::Error },

    #[snafu(display("Writer is not set"))]
    GetWriter,

    #[snafu(display("Name is not set"))]
    GetName,

    #[snafu(display("Could not get user id from message"))]
    GetUserID,

    #[snafu(display("Could not get display name from message"))]
    GetDisplayName,

    #[snafu(display("Converting user id (u64) to i64: {}", source))]
    ConvertUserID { source: std::num::TryFromIntError },
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

        let stream = twitchchat::connect_easy_tls(&name, &token)
            .await
            .context(Connect {
                server: twitchchat::TWITCH_IRC_ADDRESS_TLS,
            })?;
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
    ) -> Result<()> {
        // get a writer clone
        let mut writer = self.writer.clone().context(GetWriter)?;

        // subscribe to the events we're interested in
        let mut privmsg = dispatcher.subscribe::<events::Privmsg>();
        let mut join = dispatcher.subscribe::<events::Join>();

        // and wait for a irc ready event (blocks the current task)
        let ready = dispatcher
            .wait_for::<events::IrcReady>()
            .await
            .context(WaitForIrcReady)?;
        info!(
            "Connected to {} as {}",
            twitchchat::TWITCH_IRC_ADDRESS_TLS,
            ready.nickname
        );

        // and then join some channels
        info!("Joining twitch channels (count: {})", channels.len());
        for channel in channels {
            debug!("Joining channel (name: {})", &channel);
            writer
                .join(&channel)
                .await
                .context(JoinChannel { channel })?;
        }

        // and then our 'main loop'
        // todo: offload handling to other threads
        loop {
            tokio::select! {
                Some(msg) = privmsg.next() => {
                    trace!("Got chat message (provider: twitch, channel: {})", &msg.channel);
                    if let Err(err) = self.handle_privmsg(context.clone(), &handlers, &msg).await {
                        error!("Failed to handle privmsg: {}", err);
                    };
                },
                Some(msg) = join.next() => {
                    trace!("Got join message (provider: twitch, channel: {})", &msg.channel);
                    if let Err(err) = self.handle_join(context.clone(), &msg).await {
                        error!("Failed to handle join: {}", err);
                    };
                },
                else => break,
            }
        }

        Ok(())
    }

    async fn handle_privmsg(
        &self,
        context: Arc<BotContext>,
        handlers: &Vec<Arc<dyn Twitch>>,
        msg: &messages::Privmsg<'_>,
    ) -> Result<()> {
        // this variable name should not be changed.
        // having no name or _  as name just drops the Stopwatch instantly.
        // and having no _ infront annoys the compiler
        let _stopwatch = Stopwatch::new(|d| {
            debug!("Handling PRIVMSG took: {}", d);
        });

        trace!("Got PRIVMSG message");

        if msg.name == self.name.clone().context(GetName)? {
            // message must be sent by the bot -> ignore it
            trace!("ignoring PRIVMSG since it was sent by the bot");
            return Ok(());
        }

        // bump the user in database
        let user = {
            // todo: check all of the unwraps for errors
            let user_id = msg
                .user_id()
                .context(GetUserID)?
                .try_into()
                .context(ConvertUserID)?;
            let name = msg.name.to_owned();
            let display_name = msg.display_name().context(GetDisplayName)?;
            let now = Local::now();

            let user = User::bump(&context.conn(), user_id, &name, &display_name, &now)
                .context(BumpUser)?;

            if user.banned(&now) {
                trace!("User {} is banned. Ignoring message.", user.name);
                return Ok(());
            }

            user
        };

        for handler in handlers {
            handler.handle(Arc::new(msg.clone()), &user).await;
        }

        Ok(())
    }

    async fn handle_join(&self, context: Arc<BotContext>, msg: &messages::Join<'_>) -> Result<()> {
        if msg.channel == self.name.clone().context(GetName)? {
            // we've joined a channel
            info!("Joined {}", msg.channel);

            self.writer
                .clone()
                .context(GetWriter)?
                .privmsg(
                    &msg.channel,
                    &format!("Connected with version {}", context.version),
                )
                .await
                .context(SendPrivmsg {
                    channel: msg.channel.clone(),
                })?
        }

        Ok(())
    }

    pub fn stop(&self) {
        // get control
        let control = self.control.clone().unwrap();

        // spawn thread to stop bot
        tokio::spawn(async move { control.stop() });
    }

    pub fn join(&self, channel: &str) -> Result<()> {
        block_on(async {
            self.writer
                .clone()
                .context(GetWriter)?
                .join(channel)
                .await
                .context(JoinChannel { channel })
        })
    }

    pub fn part(&self, channel: &str) -> Result<()> {
        block_on(async {
            self.writer
                .clone()
                .context(GetWriter)?
                .part(channel)
                .await
                .context(PartChannel { channel })
        })
    }

    pub fn writer(self) -> Result<Writer> {
        self.writer.context(GetWriter)
    }
}
