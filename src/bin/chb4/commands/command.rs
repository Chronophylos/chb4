use std::fmt;
use std::sync::Arc;
use twitchchat::messages::Privmsg;

pub type CommandFunction =
    Box<dyn Fn(Vec<String>, Arc<Privmsg<'_>>) -> CommandResult + Send + Sync + 'static>;

// I want trait aliases PepeHands
// pub type CommandFunctionImpl =
//     impl Fn(Vec<String>, Arc<Privmsg<'_>>) -> CommandResult + Send + Sync + 'static;

pub struct Command {
    name: &'static str,
    aliases: Vec<&'static str>,
    chainable: bool,
    whitelisted: bool,
    description: &'static str,
    command: CommandFunction,
}

impl Command {
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn aliases(&self) -> Vec<&str> {
        self.aliases.clone()
    }

    pub fn chainable(&self) -> bool {
        self.chainable
    }

    pub fn whitelisted(&self) -> bool {
        self.whitelisted
    }

    pub fn execute(&self, args: Vec<String>, msg: Arc<Privmsg<'_>>) -> CommandResult {
        info!("Executing command {} with args {:?}", self.name, args);
        (self.command)(args, msg)
    }
}

/// Shadow constructor for `CommandBuilder`
impl Command {
    pub fn with_name(name: &'static str) -> CommandBuilder {
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
        self.name.to_owned()
    }

    fn description(&self) -> String {
        self.description.to_owned()
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

#[derive(Default)]
pub struct CommandBuilder {
    name: Option<&'static str>,
    aliases: Option<Vec<&'static str>>,
    chainable: Option<bool>,
    whitelisted: Option<bool>,
    description: Option<&'static str>,
    command: Option<CommandFunction>,
}

impl Into<Command> for CommandBuilder {
    fn into(self) -> Command {
        Command {
            name: self
                .name
                .unwrap_or_else(|| panic!("Missing name for command")),
            aliases: self.aliases.unwrap_or(Vec::new()),
            chainable: self.chainable.unwrap_or(false),
            whitelisted: self.whitelisted.unwrap_or(false),
            description: self.description.unwrap_or("description missing"),
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

    pub fn whitelisted(mut self) -> Self {
        self.whitelisted = Some(true);
        self
    }

    pub fn description(mut self, text: &'static str) -> Self {
        self.description = Some(text);
        self
    }

    pub fn command(
        mut self,
        f: impl Fn(Vec<String>, Arc<Privmsg<'_>>) -> CommandResult + Send + Sync + 'static,
    ) -> Self {
        self.command = Some(Box::new(f));
        self
    }

    pub fn done(self) -> Command {
        Command { ..self.into() }
    }
}

pub enum CommandResult {
    Message(String),
    NoMessage,
    Chainable(Vec<String>),
    Error(String),
}

impl<T> From<T> for CommandResult
where
    T: std::error::Error,
{
    fn from(error: T) -> Self {
        Self::Error(error.to_string())
    }
}
