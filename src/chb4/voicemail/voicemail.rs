use super::parser::parse_voicemail;
use chrono::prelude::*;
use nom::{error::ErrorKind, Err};
use snafu::Snafu;
use std::str::FromStr;

#[derive(Snafu, Debug)]
enum Error {
    Parser { source: Err<(String, ErrorKind)> },
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Clone)]
pub struct Voicemail {
    pub recipients: Vec<String>,
    pub message: String,
    pub schedule: Option<NaiveDateTime>,
}

impl FromStr for Voicemail {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match parse_voicemail(s) {
            Ok(t) => Ok(t.1),
            Err(e) => Err(Error::Parser {
                source: e.to_owned(),
            }),
        }
    }
}
