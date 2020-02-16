use regex::Regex;
use std::sync::Arc;
use twitchchat::messages::Privmsg;

pub struct Action {
    name: String,
    regex: Regex,
    whitelisted: bool,
    command: fn(&Arc<Privmsg<'_>>) -> ActionResult,
}

impl Action {
    pub fn execute(&self, msg: &Arc<Privmsg<'_>>) -> ActionResult {
        (self.command)(msg)
    }

    pub fn with_name(name: String) -> ActionBuilder {
        ActionBuilder::with_name(name)
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn regex(&self) -> regex::Regex {
        self.regex.clone()
    }

    pub fn whitelisted(&self) -> bool {
        self.whitelisted
    }
}

pub enum ActionResult {
    Message(String),
    NoMessage,
    Error(String),
}

pub struct ActionBuilder {
    name: String,
    regex: Regex,
    whitelisted: bool,
    command: fn(&Arc<Privmsg<'_>>) -> ActionResult,
}
impl Into<Action> for ActionBuilder {
    fn into(self) -> Action {
        Action {
            name: self.name,
            regex: Regex::new("").unwrap(),
            whitelisted: self.whitelisted,
            command: self.command,
        }
    }
}

/// Builder functions
impl ActionBuilder {
    pub fn new() -> Self {
        Self {
            name: String::from("<No Name>"),
            regex: Regex::new("").unwrap(),
            whitelisted: false,
            command: noop,
        }
    }

    pub fn with_name(name: String) -> Self {
        Self {
            name,
            ..Self::new()
        }
    }

    pub fn regex(mut self, regex: Regex) -> Self {
        self.regex = regex;
        self
    }

    pub fn whitelisted(mut self) -> Self {
        self.whitelisted = true;
        self
    }

    pub fn command(mut self, f: fn(&Arc<Privmsg<'_>>) -> ActionResult) -> Self {
        self.command = f;
        self
    }

    pub fn done(self) -> Action {
        Action { ..self.into() }
    }
}

fn noop(_msg: &Arc<Privmsg<'_>>) -> ActionResult {
    unimplemented!()
}
