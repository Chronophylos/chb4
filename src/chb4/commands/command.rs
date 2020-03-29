use crate::{
    database::User,
    helpers::prettify_bool,
    manpages::{Chapter, ManpageTrait},
    message::{Message, MessageConsumer, Result},
};
use std::{fmt, sync::Arc};

pub type CommandFunction =
    Box<dyn Fn(Vec<String>, Message, &User) -> Result + Send + Sync + 'static>;

// I want trait aliases PepeHands
// pub type CommandFunctionImpl =
//     impl Fn(Vec<String>, Message, &User) -> Result + Send + Sync + 'static;

pub struct Command {
    name: &'static str,
    aliases: Vec<&'static str>,
    chainable: bool,
    whitelisted: bool,
    about: &'static str,
    description: &'static str,
    example: Option<&'static str>,
    command: CommandFunction,
}

impl Command {
    pub fn aliases(&self) -> Vec<&str> {
        self.aliases.clone()
    }

    #[allow(dead_code)]
    pub fn chainable(&self) -> bool {
        self.chainable
    }
}

/// Shadow constructor for `CommandBuilder`
impl Command {
    pub fn with_name(name: &'static str) -> CommandBuilder {
        CommandBuilder::with_name(name)
    }
}

impl MessageConsumer for Command {
    fn name(&self) -> &str {
        self.name
    }

    fn whitelisted(&self) -> bool {
        self.whitelisted
    }

    fn consume(&self, args: Vec<String>, msg: Message, user: &User) -> Result {
        info!("Executing command {} with args {:?}", self.name, args);
        (self.command)(args, msg, &user)
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Action")
            .field("name", &self.name)
            .field("aliases", &self.aliases)
            .field("chainable", &self.chainable)
            .field("whitelisted", &self.whitelisted)
            .field("about", &self.about)
            .field("description", &self.description)
            .field("example", &self.example)
            .finish()
    }
}

impl ManpageTrait for Command {
    fn names(&self) -> Vec<&str> {
        let mut aliases = self.aliases.clone();
        aliases.insert(0, self.name);
        aliases
    }

    fn chapter(&self) -> Chapter {
        Chapter::Command
    }

    fn name(&self) -> &str {
        self.about
    }

    fn description(&self) -> &str {
        self.description
    }

    fn example(&self) -> Option<&str> {
        self.example
    }

    fn characteristics(&self) -> Vec<(&str, &str)> {
        vec![
            ("chainable", prettify_bool(self.chainable)),
            ("whitelisted", prettify_bool(self.whitelisted)),
        ]
    }
}

#[derive(Default)]
pub struct CommandBuilder {
    name: Option<&'static str>,
    aliases: Option<Vec<&'static str>>,
    chainable: Option<bool>,
    whitelisted: Option<bool>,
    about: Option<&'static str>,
    description: Option<&'static str>,
    example: Option<&'static str>,
    command: Option<CommandFunction>,
}

impl Into<Command> for CommandBuilder {
    fn into(self) -> Command {
        Command {
            name: self
                .name
                .unwrap_or_else(|| panic!("Missing name for command")),
            aliases: self.aliases.unwrap_or_default(),
            chainable: self.chainable.unwrap_or(false),
            whitelisted: self.whitelisted.unwrap_or(false),
            about: self.about.unwrap_or("about missing"),
            description: self.description.unwrap_or("description missing"),
            example: self.example,
            command: self.command.unwrap(),
        }
    }
}

/// Builder functions
impl CommandBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(name: &'static str) -> Self {
        Self {
            name: Some(name),
            ..Self::new()
        }
    }

    pub fn alias(mut self, a: &'static str) -> Self {
        if self.aliases.is_some() {
            warn!(
                "alias is used to everwrite the current alias (command: {})",
                self.name.unwrap_or("unnamed")
            )
        }

        self.aliases = Some(vec![a]);
        self
    }

    pub fn aliases(mut self, a: Vec<&'static str>) -> Self {
        if self.aliases.is_some() {
            warn!(
                "aliases is used to everwrite the current alias (command: {})",
                self.name.unwrap_or("unnamed")
            )
        }

        self.aliases = Some(a);
        self
    }

    pub fn chainable(mut self) -> Self {
        self.chainable = Some(true);
        self
    }

    #[allow(dead_code)]
    pub fn whitelisted(mut self) -> Self {
        self.whitelisted = Some(true);
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
        f: impl Fn(Vec<String>, Message, &User) -> Result + Send + Sync + 'static,
    ) -> Self {
        self.command = Some(Box::new(f));
        self
    }

    pub fn done(self) -> Arc<Command> {
        Arc::new(Command { ..self.into() })
    }
}
