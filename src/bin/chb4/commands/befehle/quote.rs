use super::prelude::*;

use chb4::database;
use chb4::helpers::Permission;
use chb4::models::User;
use regex::Regex;
use std::convert::TryInto;

pub fn command(context: Arc<Context>) -> Command {
    Command::with_name("quote")
        .alias("quotes")
        .command(move |args: Vec<String>, msg: Arc<Privmsg<'_>>| {
            let user_id = msg.user_id().unwrap().try_into().unwrap();

            let user = match database::user::by_twitch_id(&context.pool().get().unwrap(), user_id) {
                Ok(u) => u,
                Err(e) => return e.into(),
            };

            let user = match user {
                Some(u) => u,
                None => {
                    return CommandResult::Error(format!(
                        "User with twitch id {} not found",
                        user_id
                    ))
                }
            };

            match args.get(0).map(String::as_str) {
                Some("add") => add(context.clone(), msg, user, args[1..].to_vec()),
                Some("remove") | Some("delete") => remove(
                    context.clone(),
                    msg,
                    user,
                    &args.get(1).unwrap_or(&String::from("")),
                ),
                Some("edit") => edit(context.clone(), args[1..].to_vec()),
                Some("show") => show(context.clone(), &args.get(1).unwrap_or(&String::from(""))),
                Some(qid) => show(context.clone(), qid),
                None => CommandResult::Message(String::from("Missing sub-command")),
            }
        })
        .description(
            "Show or add a quote.

USAGE: quote SUBCOMMAND
       quote <quote id>

SUBCOMMANDS:
    add    - add a new quote
    remove - remove a quote
    edit   - edit a quote you made
    show   - show a quote you made
    random - show a random quote (Not Implemented)

NOTES:
    You need friend permissions to add quotes.

    When adding a quote you have to follow a specific format:
    ```
    \"<message>\" - <author> <date>
    ```
",
        )
        .done()
}

fn add(
    context: Arc<Context>,
    msg: Arc<Privmsg<'_>>,
    user: User,
    args: Vec<String>,
) -> CommandResult {
    let permission = Permission::from_user(msg.clone(), &user).unwrap();

    trace!("User permission {:?}", permission);

    if permission < Permission::Friend {
        debug!("Permission not high enough");
        return CommandResult::NoMessage;
    }

    let msg = args.join(" ");

    lazy_static! {
        static ref RE: Regex = {
            trace!("Compiling regex for `quote add`");
            Regex::new("^\"(.*)\"( - )?(.*) ([0-9:\\-\\.]+)$").unwrap()
        };
    }

    debug!("Parsing {}", msg);
    let caps = match RE.captures(&msg) {
        Some(c) => c,
        None => return CommandResult::Message(String::from("I can't parse that quote")),
    };

    debug!("Adding quote");
    let quote = match database::quote::new(
        &context.pool().get().unwrap(),
        user.id,
        caps.get(3).unwrap().as_str(), // author
        caps.get(4).unwrap().as_str(), // authored
        caps.get(1).unwrap().as_str(), // message
    ) {
        Ok(q) => q,
        Err(e) => return e.into(),
    };

    info!("Added quote {} (id: {})", quote, quote.id);
    CommandResult::Message(format!("Added new quote with id {}", quote.id))
}

fn remove(context: Arc<Context>, msg: Arc<Privmsg<'_>>, user: User, qid: &str) -> CommandResult {
    let permission = Permission::from_user(msg.clone(), &user).unwrap();

    trace!("User permission {:?}", permission);

    if permission < Permission::Friend {
        debug!("Permission not high enough");
        return CommandResult::NoMessage;
    }

    let qid: i32 = match qid.parse() {
        Ok(id) => id,
        Err(e) => return e.into(),
    };

    let quote = match database::quote::by_id(&context.pool().get().unwrap(), qid) {
        Ok(q) => q,
        Err(e) => return e.into(),
    };

    let quote = match quote {
        Some(q) => q,
        None => return CommandResult::Message(format!("Not quote with id {} found", qid)),
    };

    if quote.creator_id != user.id && permission != Permission::Owner {
        return CommandResult::Message(String::from("You do not have permissions for this quote"));
    }

    match database::quote::remove(&context.pool().get().unwrap(), qid) {
        Ok(_) => CommandResult::Message(format!("Removed quote with id {}", qid)),
        Err(e) => e.into(),
    }
}

fn edit(context: Arc<Context>, args: Vec<String>) -> CommandResult {
    unimplemented!()
}

fn show(context: Arc<Context>, qid: &str) -> CommandResult {
    let qid: i32 = match qid.parse() {
        Ok(id) => id,
        Err(e) => return e.into(),
    };

    let quote = match database::quote::by_id(&context.pool().get().unwrap(), qid) {
        Ok(q) => q,
        Err(e) => return e.into(),
    };

    match quote {
        Some(q) => CommandResult::Message(format!("{}", q)),
        None => CommandResult::Message(format!("Not quote with id {} found", qid)),
    }
}
