use crate::{
    context::BotContext,
    database::User,
    helpers::prettify_bool,
    manpages::{ChapterName, Manpage, ManpageProducer},
    message::{Message, MessageConsumer, MessageResult},
};
use anyhow::Result;
use regex::Regex;
use std::{fmt, sync::Arc};

pub type ActionFunction =
    Box<dyn Fn(Arc<BotContext>, Message, &User) -> Result<MessageResult> + Send + Sync + 'static>;

// I want trait aliases PepeHands
//pub type ActionFunctionImpl =
//    impl Fn(Arc<BotContext>, Message, &User) -> Result + Send + Sync + 'static;

pub struct Action {
    name: &'static str,
    regex: Regex,
    whitelisted: bool,
    about: &'static str,
    description: &'static str,
    example: Option<&'static str>,
    command: ActionFunction,
    noisy: bool,
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

    pub fn noisy(&self) -> bool {
        self.noisy
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
        self.name
    }

    fn whitelisted(&self) -> bool {
        self.whitelisted
    }

    fn consume(
        &self,
        context: Arc<BotContext>,
        _args: Vec<String>,
        msg: Message,
        user: &User,
    ) -> Result<MessageResult> {
        if !self.noisy {
            info!("Executing action {}", self.name);
        }
        (self.command)(context, msg, &user)
    }
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Action")
            .field("name", &self.name)
            .field("regex", &self.regex)
            .field("about", &self.name)
            .field("description", &self.description)
            .field("example", &self.example)
            .field("whitelisted", &self.whitelisted)
            .field("noisy", &self.noisy)
            .finish()
    }
}

impl ManpageProducer for Action {
    fn get_manpage(&self) -> Manpage {
        let characteristics = vec![
            (String::from("chainable"), prettify_bool(false).to_owned()),
            (
                String::from("whitelisted"),
                prettify_bool(self.whitelisted).to_owned(),
            ),
        ];

        Manpage::new(
            vec![self.name.to_owned()],
            ChapterName::Action,
            self.about.to_owned(),
            self.description.to_owned(),
            self.example.map(|s| s.to_owned()),
            characteristics,
        )
    }
}

#[derive(Default)]
pub struct ActionBuilder {
    name: Option<&'static str>,
    regex: Option<Regex>,
    whitelisted: Option<bool>,
    about: Option<&'static str>,
    description: Option<&'static str>,
    example: Option<&'static str>,
    command: Option<ActionFunction>,
    noisy: Option<bool>,
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
            about: self.about.unwrap_or("about missing"),
            description: self.description.unwrap_or("description missing"),
            example: self.example,
            command: self.command.unwrap(),
            noisy: self.noisy.unwrap_or(false),
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

    pub fn whitelisted(mut self) -> Self {
        self.whitelisted = Some(true);
        self
    }

    pub fn noisy(mut self) -> Self {
        self.noisy = Some(true);
        self
    }

    pub fn about(mut self, text: &'static str) -> Self {
        self.about = Some(text);
        self
    }

    pub fn description(mut self, text: &'static str) -> Self {
        self.description = Some(text);
        self
    }

    pub fn example(mut self, text: &'static str) -> Self {
        self.example = Some(text);
        self
    }

    pub fn command(
        mut self,
        f: impl Fn(Arc<BotContext>, Message, &User) -> Result<MessageResult> + Send + Sync + 'static,
    ) -> Self {
        self.command = Some(Box::new(f));
        self
    }

    pub fn done(self) -> Arc<Action> {
        Arc::new(Action { ..self.into() })
    }
}

#[allow(dead_code)]
pub enum ActionResult {
    Message(String),
    NoMessage,
    Error(String),
}
