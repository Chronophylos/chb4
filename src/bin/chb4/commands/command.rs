use std::fmt;
use std::sync::Arc;
use twitchchat::messages::Privmsg;

pub type CommandFunction =
    Box<dyn Fn(Vec<String>, Arc<Privmsg<'_>>) -> CommandResult + Send + Sync + 'static>;

pub struct Command {
    name: String,
    aliases: Vec<String>,
    chainable: bool,
    whitelisted: bool,
    description: String,
    command: CommandFunction,
}

impl Command {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn aliases(&self) -> Vec<String> {
        self.aliases.clone()
    }

    pub fn chainable(&self) -> bool {
        self.chainable
    }

    pub fn whitelisted(&self) -> bool {
        self.whitelisted
    }

    pub fn execute(&self, args: Vec<String>, msg: Arc<Privmsg<'_>>) -> CommandResult {
        trace!("Executing command {} with args {:?}", self.name, args);
        (self.command)(args, msg)
    }
}

/// Shadow constructor for `CommandBuilder`
impl Command {
    pub fn with_name<S: Into<String>>(name: S) -> CommandBuilder {
        CommandBuilder::with_name(name)
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Action")
            .field("name", &self.name)
            .field("aliases", &self.aliases)
            .field("chainable", &self.chainable)
            .field("whitelisted", &self.whitelisted)
            .finish()
    }
}

impl chb4::Documentation for Command {
    fn name(&self) -> String {
        self.name()
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    fn aliases(&self) -> Option<String> {
        Some(self.aliases.join(", "))
    }

    fn regex(&self) -> Option<String> {
        None
    }

    fn chainable(&self) -> String {
        String::from(if self.chainable { "yes" } else { "no" })
    }

    fn whitelisted(&self) -> String {
        String::from(if self.whitelisted { "yes" } else { "no" })
    }
}

pub struct CommandBuilder {
    name: String,
    aliases: Vec<String>,
    chainable: bool,
    whitelisted: bool,
    description: String,
    command: CommandFunction,
}

impl Into<Command> for CommandBuilder {
    fn into(self) -> Command {
        Command {
            name: self.name,
            aliases: self.aliases,
            chainable: self.chainable,
            whitelisted: self.whitelisted,
            description: self.description,
            command: self.command,
        }
    }
}

/// Builder functions
impl CommandBuilder {
    pub fn new() -> Self {
        Self {
            name: String::from("<No Name>"),
            aliases: vec![],
            chainable: false,
            whitelisted: false,
            command: Box::new(noop),
        }
    }

    pub fn with_name<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Self::new()
        }
    }

    pub fn aliases(mut self, a: Vec<String>) -> Self {
        self.aliases = a;
        self
    }

    pub fn chainable(mut self) -> Self {
        self.chainable = true;
        self
    }

    pub fn whitelisted(mut self) -> Self {
        self.whitelisted = true;
        self
    }

    pub fn description<S: Into<String>>(mut self, text: S) -> Self {
        self.description = description;
        self
    }

    pub fn command(
        mut self,
        f: impl Fn(Vec<String>, Arc<Privmsg<'_>>) -> CommandResult + Send + Sync + 'static,
    ) -> Self {
        self.command = Box::new(f);
        self
    }

    pub fn done(self) -> Command {
        Command { ..self.into() }
    }
}

fn noop(_args: Vec<String>, _msg: Arc<Privmsg<'_>>) -> CommandResult {
    panic!("Missing command when building Command")
}

pub enum CommandResult {
    Message(String),
    NoMessage,
    Chainable(Vec<String>),
    Error(String),
}
