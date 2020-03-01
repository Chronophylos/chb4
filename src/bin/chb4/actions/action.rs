use regex::Regex;
use std::fmt;
use std::sync::Arc;
use twitchchat::messages::Privmsg;

pub type ActionFunction = Box<dyn Fn(Arc<Privmsg<'_>>) -> ActionResult + Send + Sync + 'static>;

pub struct Action {
    name: String,
    regex: Regex,
    whitelisted: bool,
    description: String,
    command: ActionFunction,
}

impl Action {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn is_match(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }

    pub fn whitelisted(&self) -> bool {
        self.whitelisted
    }

    pub fn execute(&self, msg: Arc<Privmsg<'_>>) -> ActionResult {
        (self.command)(msg)
    }
}

/// Shadow constructor for `ActionBuilder`
impl Action {
    pub fn with_name<S: Into<String>>(name: S) -> ActionBuilder {
        ActionBuilder::with_name(name)
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

impl chb4::Documentation for Action {
    fn name(&self) -> String {
        self.name()
    }

    fn description(&self) -> String {
        self.description.clone()
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

pub struct ActionBuilder {
    name: String,
    regex: Regex,
    whitelisted: bool,
    description: String,
    command: ActionFunction,
}
impl Into<Action> for ActionBuilder {
    fn into(self) -> Action {
        Action {
            name: self.name,
            regex: self.regex,
            whitelisted: self.whitelisted,
            description: self.description,
            command: self.command,
        }
    }
}

/// Builder functions
impl ActionBuilder {
    pub fn new() -> Self {
        Self {
            name: String::from("<No Name>"),
            #[allow(clippy::trivial_regex)]
            regex: Regex::new("").unwrap(),
            whitelisted: false,
            description: String::new(),
            command: Box::new(noop),
        }
    }

    pub fn with_name<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Self::new()
        }
    }

    pub fn regex(mut self, regex: &str) -> Self {
        self.regex = Regex::new(regex).unwrap_or_else(|_| {
            panic!(
                "Could not parse regex ({}) when building action {}",
                regex, self.name,
            )
        });
        self
    }

    pub fn whitelisted(mut self) -> Self {
        self.whitelisted = true;
        self
    }

    pub fn description<S: Into<String>>(mut self, text: S) -> Self {
        self.description = text.into();
        self
    }

    pub fn command(
        mut self,
        f: impl Fn(Arc<Privmsg<'_>>) -> ActionResult + Send + Sync + 'static,
    ) -> Self {
        self.command = Box::new(f);
        self
    }

    pub fn done(self) -> Action {
        Action { ..self.into() }
    }
}

fn noop(_msg: Arc<Privmsg<'_>>) -> ActionResult {
    panic!("Missing command when building Action")
}

pub enum ActionResult {
    Message(String),
    NoMessage,
    Error(String),
}
