use super::{Connection, User};
use crate::schema::*;
use chrono::prelude::*;
use diesel::prelude::*;
use snafu::{ResultExt, Snafu};
use std::{
    fmt,
    fmt::{Display, Formatter},
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Inserting quote: {}", source))]
    InsertQuote { source: diesel::result::Error },

    #[snafu(display("Getting quote (id: {}): {}", id, source))]
    GetQuoteByID {
        id: i32,
        source: diesel::result::Error,
    },

    #[snafu(display("Removing quote (id: {}): {}", id, source))]
    RemoveQuote {
        id: i32,
        source: diesel::result::Error,
    },

    #[snafu(display("Updating quote (id: {}): {}", id, source))]
    UpdateQuote {
        id: i32,
        source: diesel::result::Error,
    },
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Queryable, Identifiable, Associations)]
#[belongs_to(User, foreign_key = "creator_id")]
#[table_name = "quotes"]
pub struct Quote {
    pub id: i32,
    pub creator_id: i32,
    pub created: NaiveDateTime,
    pub author: String,
    pub authored: String,
    pub message: String,
}

impl Display for Quote {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\"{}\" - {} {}",
            self.message, self.author, self.authored
        )
    }
}

impl Quote {
    pub fn new<'a>(
        conn: &Connection,
        creator_id: i32,
        author: &'a str,
        authored: &'a str,
        message: &'a str,
    ) -> Result<Self> {
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

    pub fn by_id(conn: &Connection, id: i32) -> Result<Option<Self>> {
        trace!("Getting quote (id: {})", id);

        quotes::table
            .filter(quotes::id.eq(id))
            .get_result(conn)
            .optional()
            .context(GetQuoteByID { id })
    }

    pub fn update<'a>(
        &self,
        conn: &Connection,
        author: &'a str,
        authored: &'a str,
        message: &'a str,
    ) -> Result<Self> {
        trace!("Updating quote (id: {})", self.id);

        let quote = diesel::update(self)
            .set(&EditQuote {
                author,
                authored,
                message,
            })
            .get_result(conn)
            .context(UpdateQuote { id: self.id })?;

        Ok(quote)
    }

    pub fn remove(&self, conn: &Connection) -> Result<()> {
        trace!("Removing quote (id: {})", self.id);

        diesel::delete(self)
            .execute(conn)
            .context(RemoveQuote { id: self.id })?;

        Ok(())
    }
}

#[derive(Insertable)]
#[table_name = "quotes"]
pub struct NewQuote<'a> {
    pub creator_id: i32,
    pub created: &'a NaiveDateTime,
    pub author: &'a str,
    pub authored: &'a str,
    pub message: &'a str,
}

#[derive(AsChangeset)]
#[table_name = "quotes"]
pub struct EditQuote<'a> {
    pub author: &'a str,
    pub authored: &'a str,
    pub message: &'a str,
}
