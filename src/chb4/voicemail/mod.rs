use chrono::prelude::*;

#[derive(Debug, PartialEq)]
pub struct Voicemail {
    recipients: Vec<String>,
    message: String,
    schedule: Option<NaiveDateTime>,
}

pub mod parser;
