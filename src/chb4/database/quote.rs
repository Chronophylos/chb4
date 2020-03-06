use crate::models::{NewQuote, Quote};
use crate::schema::*;
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::PgConnection;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    InsertQuote { source: diesel::result::Error },
    GetQuoteByID { source: diesel::result::Error },
    RemoveQuote { source: diesel::result::Error },
}

type Result<T> = std::result::Result<T, Error>;

pub fn new<'a>(
    conn: &PgConnection,
    creator_id: i32,
    author: &'a str,
    authored: &'a str,
    message: &'a str,
) -> Result<Quote> {
    trace!("Creating new quote");

    let quote = diesel::insert_into(quotes::table)
        .values(&NewQuote {
            creator_id,
            created: &Utc::now().naive_utc(),
            author,
            authored,
            message,
        })
        .get_result(conn)
        .context(InsertQuote)?;

    Ok(quote)
}

pub fn by_id(conn: &PgConnection, id: i32) -> Result<Option<Quote>> {
    trace!("Getting quote (id: {})", id);

    quotes::table
        .filter(quotes::id.eq(id))
        .get_result::<Quote>(conn)
        .optional()
        .context(GetQuoteByID)
}

pub fn remove(conn: &PgConnection, id: i32) -> Result<()> {
    trace!("Removing quote (id: {})", id);

    diesel::delete(quotes::table)
        .filter(quotes::id.eq(id))
        .execute(conn)
        .context(RemoveQuote)?;

    Ok(())
}
