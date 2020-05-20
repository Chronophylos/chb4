use super::prelude::*;

use chb4::database::{Quote, User};
use chb4::helpers::Permission;
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

fn add(context: Arc<BotContext>, msg: Message, user: &User, args: Vec<String>) -> Result {
    let permission = Permission::from_user(msg, &user).unwrap();

    // check if permission is at least friend
    if permission < Permission::Friend {
        debug!("Permission not high enough");
        return Ok(MessageResult::None);
    }

    let msg = args.join(" ");
    let (message, author, authored) = match parse_quote(&msg) {
        Ok(t) => t,
        Err(e) => return Err(e),
    };

    // insert quote
    let quote = match Quote::new(&context.conn(), user.id, author, authored, message) {
        Ok(q) => q,
        Err(e) => return Err(MessageError::from(e.to_string())),
    };

    info!("Added quote {} (id: {})", quote, quote.id);
    Ok(MessageResult::Message(format!(
        "Added new quote with id {}",
        quote.id
    )))
}

fn remove(context: Arc<BotContext>, msg: Message, user: &User, qid: Option<&str>) -> Result {
    // unwrap option
    let qid: &str = match qid {
        Some(qid) => qid,
        None => {
            return Ok(MessageResult::Message(String::from(
                "Argument quote id missing",
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
    let qid: i32 = match qid.parse() {
        Ok(id) => id,
        Err(e) => return Err(MessageError::from(e.to_string())),
    };

    let conn = &context.conn();

    // query quote
    let quote = match Quote::by_id(conn, qid) {
        Ok(q) => q,
        Err(e) => return Err(MessageError::from(e.to_string())),
    };

    let quote = match quote {
        Some(q) => q,
        None => {
            return Ok(MessageResult::Message(format!(
                "Not quote with id {} found",
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

    match quote.remove(conn) {
        Ok(_) => Ok(MessageResult::Message(format!(
            "Removed quote with id {}",
            qid
        ))),
        Err(e) => Err(MessageError::from(e.to_string())),
    }
}

fn edit(context: Arc<BotContext>, msg: Message, user: &User, args: Vec<String>) -> Result {
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
    let qid: i32 = match qid.parse() {
        Ok(id) => id,
        Err(e) => return Err(MessageError::from(e.to_string())),
    };

    let conn = &context.conn();

    // query quote
    let quote = match Quote::by_id(conn, qid) {
        Ok(q) => q,
        Err(e) => return Err(MessageError::from(e.to_string())),
    };

    let quote = match quote {
        Some(q) => q,
        None => {
            return Ok(MessageResult::Message(format!(
                "Not quote with id {} found",
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
        Err(e) => return Err(e),
    };

    let quote = match quote.update(conn, author, authored, message) {
        Ok(q) => q,
        Err(e) => return Err(MessageError::from(e.to_string())),
    };

    Ok(MessageResult::Message(format!("Edited quote {}", quote.id)))
}

fn show(context: Arc<BotContext>, qid: Option<&str>) -> Result {
    // unwrap option
    let qid: &str = match qid {
        Some(qid) => qid,
        None => {
            return Ok(MessageResult::Message(String::from(
                "Argument quote id missing",
            )))
        }
    };

    // parse str to i32
    let qid: i32 = match qid.parse() {
        Ok(id) => id,
        Err(e) => return Err(MessageError::from(e.to_string())),
    };

    // query quote
    let quote = match Quote::by_id(&context.conn(), qid) {
        Ok(q) => q,
        Err(e) => return Err(MessageError::from(e.to_string())),
    };

    Ok(MessageResult::Message(match quote {
        Some(q) => format!("{}", q),
        None => format!("Not quote with id {} found", qid),
    }))
}

fn parse_quote(msg: &str) -> std::result::Result<(&str, &str, &str), MessageError> {
    // parse quote
    let caps = match RE.captures(&msg) {
        Some(c) => c,
        None => return Err("I can't parse that quote".into()),
    };

    let message = caps.get(1).unwrap().as_str();
    let author = caps.get(3).unwrap().as_str();
    let authored = caps.get(4).unwrap().as_str();

    if message.len() > 500 {
        return Err("Quote is too long, max length is 500".into());
    }

    if author.len() > 25 {
        return Err("Author is too long, max length is 25".into());
    }

    if authored.len() > 25 {
        return Err("Date is too long, max length is 25".into());
    }

    Ok((message, author, authored))
}
