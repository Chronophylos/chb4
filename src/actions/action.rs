use regex::Regex;
use std::sync::Arc;
use twitchchat::messages::Privmsg;

pub struct Action<'a> {
    name: &'a str,
    regex: Regex,
    whitelisted: bool,
    command: fn(&Arc<Privmsg<'_>>) -> ActionResult<'a>,
}

impl<'a> Action<'a> {
    pub fn execute(&self, msg: &Arc<Privmsg<'_>>) -> ActionResult {
        (self.command)(msg)
    }

    pub fn with_name(name: &'a str) -> ActionBuilder<'a> {
        ActionBuilder::with_name(name)
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn regex(&self) -> regex::Regex {
        self.regex.clone()
    }

    pub fn whitelisted(&self) -> bool {
        self.whitelisted
    }
}

#[allow(dead_code)]
pub enum ActionResult<'a> {
    Message(&'a str),
    NoMessage,
    Error(&'a str),
}

pub struct ActionBuilder<'a> {
    name: &'a str,
    regex: Regex,
    whitelisted: bool,
    command: fn(&Arc<Privmsg<'_>>) -> ActionResult<'a>,
}
impl<'a> Into<Action<'a>> for ActionBuilder<'a> {
    fn into(self) -> Action<'a> {
        Action {
            name: self.name,
            regex: Regex::new("").unwrap(),
            whitelisted: self.whitelisted,
            command: self.command,
        }
    }
}

/// Builder functions
impl<'a> ActionBuilder<'a> {
    pub fn new() -> Self {
        Self {
            name: "<No Name>",
            regex: Regex::new("").unwrap(),
            whitelisted: false,
            command: noop,
        }
    }

    pub fn with_name(name: &'a str) -> Self {
        Self {
            name,
            ..Self::new()
        }
    }

    pub fn regex(mut self, regex: &str) -> Self {
        self.regex = Regex::new(regex).expect(&format!(
            "Could not parse regex ({}) when building action {}",
            regex, self.name,
        ));
        self
    }

    #[allow(dead_code)]
    pub fn whitelisted(mut self) -> Self {
        self.whitelisted = true;
        self
    }

    pub fn command(mut self, f: fn(&Arc<Privmsg<'_>>) -> ActionResult<'a>) -> Self {
        self.command = f;
        self
    }

    pub fn done(self) -> Action<'a> {
        Action { ..self.into() }
    }
}

fn noop<'a>(_msg: &Arc<Privmsg<'_>>) -> ActionResult<'a> {
    unimplemented!()
}