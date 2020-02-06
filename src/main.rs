#[macro_use]
extern crate log;
extern crate flexi_logger;
extern crate yansi;

#[tokio::main]
async fn main() {
    flexi_logger::Logger::with_env()
        .format(custom_log_format)
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    use futures::prelude::*;

    let (nick, pass) = (
        // twitch name
        std::env::var("TWITCH_NICK").unwrap(),
        // oauth token for twitch name
        std::env::var("TWITCH_PASS").unwrap(),
    );

    // putting this in the env so people don't join my channel when running this
    let channel = std::env::var("TWITCH_CHANNEL").unwrap();

    // connect via (tls or normal, 'Secure' determines that) tcp with this nick and password
    let (read, write) = twitchchat::connect_easy(&nick, &pass, twitchchat::Secure::Nope)
        .await
        .unwrap();

    info!("creating new client");

    trace!("trace");
    debug!("debug");
    info!("info");
    warn!("warn");
    error!("error");

    // make a client. the client is clonable
    let client = twitchchat::Client::new();

    // get a future that resolves when the client is done reading, fails to read/write or is stopped
    let done = client.run(read, write);

    // get an event dispatcher
    let mut dispatcher = client.dispatcher().await;

    // subscribe to an event stream

    // for privmsg (what users send to channels)
    let mut privmsg = dispatcher.subscribe::<twitchchat::events::Privmsg>();
    // spawn a task to consume the stream
    tokio::task::spawn(async move {
        while let Some(msg) = privmsg.next().await {
            trace!("Got PRIVMSG");
            info!("[{}] {}: {}", msg.channel, msg.name, msg.data);
        }
    });

    // for join (when a user joins a channel)
    let mut join = dispatcher.subscribe::<twitchchat::events::Join>();
    tokio::task::spawn(async move {
        while let Some(msg) = join.next().await {
            trace!("Got JOIN");
            // we've joined a channel
            if msg.name == nick {
                info!("you joined {}", msg.channel);
                break; // returning/dropping the stream un-subscribes it
            }
        }
    });

    // for privmsg again
    let mut bot = dispatcher.subscribe::<twitchchat::events::Privmsg>();
    // we can move the client to another task by cloning it
    //?? why should I do this and what does it do?
    let bot_client = client.clone();
    tokio::task::spawn(async move {
        let mut writer = bot_client.writer();
        while let Some(msg) = bot.next().await {
            match msg.data.split(" ").next() {
                Some("!quit") => {
                    // causes the client to shutdown
                    bot_client.stop().await.unwrap();
                }
                Some("!hello") => {
                    let response = format!("hello {}!", msg.name);
                    // send a message in response
                    if let Err(_err) = writer.privmsg(&msg.channel, &response).await {
                        // we ran into a write error, we should probably leave this task
                        return;
                    }
                }
                _ => {}
            }
        }
    });

    // dispatcher has an RAII guard, so keep it scoped
    // dropping it here so everything can proceed while keeping example brief
    drop(dispatcher);
    trace!("dropped dispatcher");

    info!("joining channel");
    // get a clonable writer from the client
    // join a channel, methods on writer return false if the client is disconnected
    if let Err(err) = client.writer().join(&channel).await {
        match err {
            twitchchat::Error::InvalidChannel(..) => {
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
        Ok(twitchchat::client::Status::Eof) => {
            info!("done!");
        }
        Ok(twitchchat::client::Status::Canceled) => {
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
fn custom_log_format(
    w: &mut dyn std::io::Write,
    now: &mut flexi_logger::DeferredNow,
    record: &flexi_logger::Record,
) -> Result<(), std::io::Error> {
    let level = record.level();
    write!(
        w,
        " {} {:<5} {} > {}",
        now.now().format("%Y-%m-%d %H:%M:%S"),
        style_level(level, record.level()),
        yansi::Paint::new(record.module_path().unwrap_or("<unnamed>")).bold(),
        style_message(level, &record.args())
    )
}
fn style_level<T>(level: log::Level, item: T) -> yansi::Paint<T> {
    match level {
        log::Level::Error => yansi::Paint::red(item).bold(),
        log::Level::Warn => yansi::Paint::yellow(item).bold(),
        log::Level::Info => yansi::Paint::green(item),
        log::Level::Debug => yansi::Paint::blue(item),
        log::Level::Trace => yansi::Paint::fixed(5, item),
    }
}
fn style_message<T>(level: log::Level, item: T) -> yansi::Paint<T> {
    match level {
        log::Level::Error => yansi::Paint::red(item),
        log::Level::Warn => yansi::Paint::yellow(item),
        log::Level::Info => yansi::Paint::white(item),
        log::Level::Debug => yansi::Paint::fixed(8, item),
        log::Level::Trace => yansi::Paint::fixed(8, item).italic(),
    }
}
