pub struct Command<'a> {
    name: &'a str,
    aliases: Vec<&'a str>,
    #[allow(dead_code)]
    chainable: bool,
    whitelisted: bool,
    command: fn(Vec<&str>) -> CommandResult,
}

impl<'a> Command<'a> {
    pub fn execute(&self, args: Vec<&str>) -> CommandResult {
        trace!("Executing command {} with args {:?}", self.name, args);
        (self.command)(args)
    }

    pub fn name(&self) -> String {
        self.name.to_string()
    }

    pub fn aliases(&self) -> Vec<&str> {
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
impl<'a> Command<'a> {
    pub fn with_name(name: &'a str) -> CommandBuilder<'a> {
        CommandBuilder::with_name(name)
    }
}

pub struct CommandBuilder<'a> {
    name: &'a str,
    aliases: Vec<&'a str>,
    chainable: bool,
    whitelisted: bool,
    command: fn(Vec<&str>) -> CommandResult,
}

impl<'a> Into<Command<'a>> for CommandBuilder<'a> {
    fn into(self) -> Command<'a> {
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
impl<'a> CommandBuilder<'a> {
    pub fn new() -> Self {
        Self {
            name: "<No Name>",
            aliases: vec![],
            chainable: false,
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

    pub fn command(mut self, f: fn(Vec<&str>) -> CommandResult) -> Self {
        self.command = f;
        self
    }

    pub fn aliases(mut self, a: Vec<&'a str>) -> Self {
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

    pub fn done(self) -> Command<'a> {
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
