use super::prelude::*;

use chb4::database;
use chb4::helpers::Permission;
use chb4::models::User;
use regex::Regex;
use std::convert::TryInto;

lazy_static! {
    static ref RE: Regex = {
        trace!("Compiling regex for `quote`");
        Regex::new(r#"^"(.*)"( - )?(.*) ([0-9:\-\.]+)$"#).unwrap()
    };
}

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
                Some("remove") | Some("delete") => {
                    remove(context.clone(), msg, user, args.get(1).map(String::as_str))
                }
                Some("edit") => edit(context.clone(), msg, user, args[1..].to_vec()),
                Some("show") => show(context.clone(), args.get(1).map(String::as_str)),
                Some(qid) => show(context.clone(), Some(qid)),
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

    // check if permission is at least friend
    if permission < Permission::Friend {
        debug!("Permission not high enough");
        return CommandResult::NoMessage;
    }

    let msg = args.join(" ");
    let (message, author, authored) = match parse_quote(&msg) {
        Result::Ok(t) => t,
        Result::Err(e) => return CommandResult::Message(e),
    };

    // insert quote
    let quote = match database::quote::new(
        &context.pool().get().unwrap(),
        user.id,
        author,
        authored,
        message,
    ) {
        Ok(q) => q,
        Err(e) => return e.into(),
    };

    info!("Added quote {} (id: {})", quote, quote.id);
    CommandResult::Message(format!("Added new quote with id {}", quote.id))
}

fn remove(
    context: Arc<Context>,
    msg: Arc<Privmsg<'_>>,
    user: User,
    qid: Option<&str>,
) -> CommandResult {
    // unwrap option
    let qid: &str = match qid {
        Some(qid) => qid,
        None => return CommandResult::Message(String::from("Argument quote id missing")),
    };

    let permission = Permission::from_user(msg.clone(), &user).unwrap();

    // check if permission is at least friend
    if permission < Permission::Friend {
        debug!("Permission not high enough");
        return CommandResult::NoMessage;
    }

    // parse str to i32
    let qid: i32 = match qid.parse() {
        Ok(id) => id,
        Err(e) => return e.into(),
    };

    // query quote
    let quote = match database::quote::by_id(&context.pool().get().unwrap(), qid) {
        Ok(q) => q,
        Err(e) => return e.into(),
    };

    let quote = match quote {
        Some(q) => q,
        None => return CommandResult::Message(format!("Not quote with id {} found", qid)),
    };

    // check permissions
    if quote.creator_id != user.id && permission != Permission::Owner {
        return CommandResult::Message(String::from("You do not have permissions for this quote"));
    }

    match database::quote::remove(&context.pool().get().unwrap(), qid) {
        Ok(_) => CommandResult::Message(format!("Removed quote with id {}", qid)),
        Err(e) => e.into(),
    }
}

fn edit(
    context: Arc<Context>,
    msg: Arc<Privmsg<'_>>,
    user: User,
    args: Vec<String>,
) -> CommandResult {
    // unwrap option
    let qid: &str = match args.get(0) {
        Some(q) => q,
        None => return CommandResult::Message(String::from("Missing quote id")),
    };

    let permission = Permission::from_user(msg.clone(), &user).unwrap();

    // check if permission is at least friend
    if permission < Permission::Friend {
        debug!("Permission not high enough");
        return CommandResult::NoMessage;
    }

    // parse str to i32
    let qid: i32 = match qid.parse() {
        Ok(id) => id,
        Err(e) => return e.into(),
    };

    // query quote
    let quote = match database::quote::by_id(&context.pool().get().unwrap(), qid) {
        Ok(q) => q,
        Err(e) => return e.into(),
    };

    let quote = match quote {
        Some(q) => q,
        None => return CommandResult::Message(format!("Not quote with id {} found", qid)),
    };

    // check permissions
    if quote.creator_id != user.id && permission != Permission::Owner {
        return CommandResult::Message(String::from("You do not have permissions for this quote"));
    }

    let msg = args.join(" ");
    let (message, author, authored) = match parse_quote(&msg) {
        Result::Ok(t) => t,
        Result::Err(e) => return CommandResult::Message(e),
    };

    let quote = match database::quote::update(
        &context.pool().get().unwrap(),
        &quote,
        author,
        authored,
        message,
    ) {
        Ok(q) => q,
        Err(e) => return e.into(),
    };

    CommandResult::Message(format!("Edited quote {}", quote.id))
}

fn show(context: Arc<Context>, qid: Option<&str>) -> CommandResult {
    // unwrap option
    let qid: &str = match qid {
        Some(qid) => qid,
        None => return CommandResult::Message(String::from("Argument quote id missing")),
    };

    // parse str to i32
    let qid: i32 = match qid.parse() {
        Ok(id) => id,
        Err(e) => return e.into(),
    };

    // query quote
    let quote = match database::quote::by_id(&context.pool().get().unwrap(), qid) {
        Ok(q) => q,
        Err(e) => return e.into(),
    };

    match quote {
        Some(q) => CommandResult::Message(format!("{}", q)),
        None => CommandResult::Message(format!("Not quote with id {} found", qid)),
    }
}

fn parse_quote(msg: &str) -> Result<(&str, &str, &str)> {
    // parse quote
    let caps = match RE.captures(&msg) {
        Some(c) => c,
        None => return Result::Err(String::from("I can't parse that quote")),
    };

    let message = caps.get(1).unwrap().as_str();
    let author = caps.get(3).unwrap().as_str();
    let authored = caps.get(4).unwrap().as_str();

    if message.len() > 500 {
        return Result::Err(String::from("Quote is too long, max length is 500"));
    }

    if author.len() > 25 {
        return Result::Err(String::from("Author is too long, max length is 25"));
    }

    if authored.len() > 25 {
        return Result::Err(String::from("Date is too long, max length is 25"));
    }

    Result::Ok((message, author, authored))
}
