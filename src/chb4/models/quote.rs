use crate::models::User;
use crate::schema::*;
use chrono::prelude::*;
use std::fmt;

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

impl fmt::Display for Quote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\"{}\" - {} {}",
            self.message, self.author, self.authored
        )
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
