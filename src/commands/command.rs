pub struct Command {
    name: String,
    aliases: Vec<String>,
    #[allow(dead_code)]
    chainable: bool,
    whitelisted: bool,
    command: fn(Vec<&str>) -> CommandResult,
}

impl Command {
    pub fn execute(&self, args: Vec<&str>) -> CommandResult {
        trace!("Executing command {} with args {:?}", self.name, args);
        (self.command)(args)
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn aliases(&self) -> Vec<String> {
        self.aliases.clone()
    }

    #[allow(dead_code)]
    pub fn chainable(&self) -> bool {
        self.chainable
    }

    pub fn whitelisted(&self) -> bool {
        self.whitelisted
    }
}

/// Shadow constructors for `CommandBuilder`
impl Command {
    pub fn with_name(name: String) -> CommandBuilder {
        CommandBuilder::with_name(name)
    }
}

pub struct CommandBuilder {
    name: String,
    aliases: Vec<String>,
    chainable: bool,
    whitelisted: bool,
    command: fn(Vec<&str>) -> CommandResult,
}

impl Default for CommandBuilder {
    fn default() -> Self {
        Self {
            name: String::from("<No Name>"),
            aliases: vec![],
            chainable: false,
            whitelisted: false,
            command: noop,
        }
    }
}

impl Into<Command> for CommandBuilder {
    fn into(self) -> Command {
        Command {
            name: self.name,
            aliases: self.aliases,
            chainable: self.chainable,
            whitelisted: self.whitelisted,
            command: self.command,
        }
    }
}

#[allow(dead_code)]
/// Builder functions
impl CommandBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(name: String) -> Self {
        Self {
            name,
            ..Self::new()
        }
    }

    pub fn command(mut self, f: fn(Vec<&str>) -> CommandResult) -> Self {
        self.command = f;
        self
    }

    pub fn aliases(mut self, a: Vec<String>) -> Self {
        self.aliases = a;
        self
    }

    pub fn chainable(mut self, c: bool) -> Self {
        self.chainable = c;
        self
    }

    pub fn whitelisted(mut self, w: bool) -> Self {
        self.whitelisted = w;
        self
    }

    pub fn done(self) -> Command {
        Command { ..self.into() }
    }
}

fn noop(_args: Vec<&str>) -> CommandResult {
    unimplemented!()
}

#[allow(dead_code)]
pub enum CommandResult {
    Message(String),
    Chainable(Vec<String>),
    Error(String),
}
