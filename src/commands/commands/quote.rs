use super::prelude::*;

use crate::database::{Quote, User};
use crate::helpers::Permission;
use regex::Regex;

lazy_static! {
    static ref RE: Regex = {
        trace!("Compiling regex for `quote`");
        Regex::new(r#"^"(.*)"( - )?(.*) ([0-9:\-\.]+)$"#).unwrap()
    };
}

pub fn command() -> Arc<Command> {
    Command::with_name("quote")
        .alias("quotes")
        .command(
            move |context, args, msg, user| match args.get(0).map(String::as_str) {
                Some("add") => add(context.clone(), msg, user, args[1..].to_vec()),
                Some("remove") | Some("delete") => {
                    remove(context.clone(), msg, user, args.get(1).map(String::as_str))
                }
                Some("edit") => edit(context.clone(), msg, user, args[1..].to_vec()),
                Some("show") => show(context.clone(), args.get(1).map(String::as_str)),
                Some(qid) => show(context.clone(), Some(qid)),
                None => Ok(MessageResult::Message(String::from("Missing sub-command"))),
            },
        )
        .about("Show or manage quotes")
        .description(
            "
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
    context: Arc<BotContext>,
    msg: Message,
    user: &User,
    args: Vec<String>,
) -> Result<MessageResult> {
    let permission = Permission::from_user(msg, &user).unwrap();

    // check if permission is at least friend
    if permission < Permission::Friend {
        debug!("Permission not high enough");
        return Ok(MessageResult::None);
    }

    let msg = args.join(" ");
    let (message, author, authored) = match parse_quote(&msg) {
        Ok(t) => t,
        Err(err) => return Ok(MessageResult::Error(err.to_string())),
    };

    // insert quote
    let quote = Quote::new(&context.conn(), user.id, author, authored, message)?;

    info!("Added quote {} (id: {})", quote, quote.id);
    Ok(MessageResult::Message(format!(
        "Added new quote with id {}",
        quote.id
    )))
}

fn remove(
    context: Arc<BotContext>,
    msg: Message,
    user: &User,
    qid: Option<&str>,
) -> Result<MessageResult> {
    // unwrap option
    let qid: &str = match qid {
        Some(qid) => qid,
        None => {
            return Ok(MessageResult::Error(String::from(
                "Missing argument: quote id",
            )))
        }
    };

    let permission = Permission::from_user(msg, &user).unwrap();

    // check if permission is at least friend
    if permission < Permission::Friend {
        debug!("Permission not high enough");
        return Ok(MessageResult::None);
    }

    // parse str to i32
    let qid: i32 = qid.parse()?;

    let conn = &context.conn();

    // query quote
    let quote = Quote::by_id(conn, qid)?;

    let quote = match quote {
        Some(q) => q,
        None => {
            return Ok(MessageResult::Message(format!(
                "No quote with id {} found",
                qid
            )))
        }
    };

    // check permissions
    if quote.creator_id != user.id && permission != Permission::Owner {
        return Ok(MessageResult::Message(String::from(
            "You do not have permissions for this quote",
        )));
    }

    quote.remove(conn)?;

    Ok(MessageResult::Message(format!(
        "Removed quote with id {}",
        qid
    )))
}

fn edit(
    context: Arc<BotContext>,
    msg: Message,
    user: &User,
    args: Vec<String>,
) -> Result<MessageResult> {
    // unwrap option
    let qid: &str = match args.get(0) {
        Some(q) => q,
        None => return Ok(MessageResult::Message(String::from("Missing quote id"))),
    };

    let permission = Permission::from_user(msg, &user).unwrap();

    // check if permission is at least friend
    if permission < Permission::Friend {
        debug!("Permission not high enough");
        return Ok(MessageResult::None);
    }

    // parse str to i32
    let qid: i32 = qid.parse()?;

    let conn = &context.conn();

    // query quote
    let quote = Quote::by_id(conn, qid)?;

    let quote = match quote {
        Some(q) => q,
        None => {
            return Ok(MessageResult::Message(format!(
                "No quote with id {} found",
                qid
            )))
        }
    };

    // check permissions
    if quote.creator_id != user.id && permission != Permission::Owner {
        return Ok(MessageResult::Message(String::from(
            "You do not have permissions for this quote",
        )));
    }

    let msg = args.join(" ");
    let (message, author, authored) = match parse_quote(&msg) {
        Ok(t) => t,
        Err(err) => return Ok(MessageResult::Error(err.to_string())),
    };

    let quote = quote.update(conn, author, authored, message)?;

    Ok(MessageResult::Message(format!(
        "Updated quote {}",
        quote.id
    )))
}

fn show(context: Arc<BotContext>, qid: Option<&str>) -> Result<MessageResult> {
    // unwrap option
    let qid: &str = match qid {
        Some(qid) => qid,
        None => {
            return Ok(MessageResult::Error(String::from(
                "Missing argument: quote id",
            )))
        }
    };

    // parse str to i32
    let qid: i32 = qid.parse()?;

    // query quote
    let quote = Quote::by_id(&context.conn(), qid)?;

    Ok(MessageResult::Message(match quote {
        Some(q) => format!("{}", q),
        None => format!("No quote with id {} found", qid),
    }))
}

fn parse_quote(msg: &str) -> Result<(&str, &str, &str)> {
    // parse quote
    let caps = RE.captures(&msg).context("Regex does not match")?;

    let message = caps.get(1).unwrap().as_str();
    let author = caps.get(3).unwrap().as_str();
    let authored = caps.get(4).unwrap().as_str();

    ensure!(message.len() < 400, "Quote is too long, max lenght is 400");
    ensure!(
        author.len() < 400,
        "Author name is too long, max lenght is 25"
    );
    ensure!(authored.len() < 400, "Date is too long, max lenght is 25");

    Ok((message, author, authored))
}
