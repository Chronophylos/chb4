use chrono::prelude::*;

#[derive(Debug, PartialEq)]
pub struct Voicemail<'a> {
    recipients: Vec<&'a str>,
    message: &'a str,
    schedule: Option<NaiveDateTime>,
}

pub mod parser;
