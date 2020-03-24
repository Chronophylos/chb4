use crate::{
    database::User,
    documentation::Documentation,
    message::{Message, MessageConsumer, Result},
};
use regex::Regex;
use std::fmt;

pub type ActionFunction = Box<dyn Fn(Message, &User) -> Result + Send + Sync + 'static>;

// I want trait aliases PepeHands
// pub type ActionFunctionImpl =
//     impl Fn(Message, &User) -> Result + Send + Sync + 'static;

pub struct Action {
    name: &'static str,
    regex: Regex,
    whitelisted: bool,
    description: &'static str,
    command: ActionFunction,
}

impl Action {
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn is_match(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }

    pub fn whitelisted(&self) -> bool {
        self.whitelisted
    }
}

/// Shadow constructor for `ActionBuilder`
impl Action {
    pub fn with_name(name: &'static str) -> ActionBuilder {
        ActionBuilder::with_name(name)
    }
}

impl MessageConsumer for Action {
    fn name(&self) -> &str {
        self.name()
    }

    fn whitelisted(&self) -> bool {
        self.whitelisted()
    }

    fn consume(&self, _args: Vec<String>, msg: Message, user: &User) -> Result {
        info!("Executing action {}", self.name);
        (self.command)(msg, &user)
    }
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Action")
            .field("name", &self.name)
            .field("regex", &self.regex)
            .field("whitelisted", &self.whitelisted)
            .finish()
    }
}

impl Documentation for Action {
    fn name(&self) -> String {
        self.name().to_owned()
    }

    fn description(&self) -> String {
        self.description.to_owned()
    }

    fn aliases(&self) -> Option<String> {
        None
    }

    fn regex(&self) -> Option<String> {
        Some(format!("{}", self.regex))
    }

    fn chainable(&self) -> String {
        String::from("no")
    }

    fn whitelisted(&self) -> String {
        String::from(if self.whitelisted { "yes" } else { "no" })
    }
}

#[derive(Default)]
pub struct ActionBuilder {
    name: Option<&'static str>,
    regex: Option<Regex>,
    whitelisted: Option<bool>,
    description: Option<&'static str>,
    command: Option<ActionFunction>,
}

impl Into<Action> for ActionBuilder {
    fn into(self) -> Action {
        Action {
            name: self
                .name
                .unwrap_or_else(|| panic!("Missing name for command")),
            regex: self.regex.unwrap_or_else(|| {
                #[allow(clippy::trivial_regex)]
                Regex::new("").unwrap()
            }),
            whitelisted: self.whitelisted.unwrap_or(false),
            description: self.description.unwrap_or("description missing"),
            command: self.command.unwrap(),
        }
    }
}

/// Builder functions
impl ActionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(name: &'static str) -> Self {
        Self {
            name: Some(name),
            ..Self::new()
        }
    }

    pub fn regex(mut self, regex: &str) -> Self {
        self.regex =
            Some(Regex::new(regex).unwrap_or_else(|_| {
                panic!("Could not parse regex ({}) when building action", regex)
            }));
        self
    }

    #[allow(dead_code)]
    pub fn whitelisted(mut self) -> Self {
        self.whitelisted = Some(true);
        self
    }

    #[allow(dead_code)]
    pub fn description(mut self, text: &'static str) -> Self {
        self.description = Some(text);
        self
    }

    pub fn command(mut self, f: impl Fn(Message, &User) -> Result + Send + Sync + 'static) -> Self {
        self.command = Some(Box::new(f));
        self
    }

    pub fn done(self) -> Action {
        Action { ..self.into() }
    }
}

#[allow(dead_code)]
pub enum ActionResult {
    Message(String),
    NoMessage,
    Error(String),
}
