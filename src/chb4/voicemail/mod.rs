use chrono::prelude::*;
use std::str::FromStr;

mod parser;

#[derive(Debug, PartialEq, Clone)]
pub struct Voicemail {
    pub recipients: Vec<String>,
    pub message: String,
    pub schedule: Option<NaiveDateTime>,
}

#[derive(Debug)]
pub struct Error(String);

impl FromStr for Voicemail {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parser::parse_voicemail(s) {
            Ok(t) => Ok(t.1),
            Err(t) => Err(Error(format!("{:?}", t))),
        }
    }
}
