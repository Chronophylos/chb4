use std::error;
use std::fmt;

pub struct Command {
    pub name: String,
    pub aliases: Vec<String>,
    #[allow(dead_code)]
    pub chainable: bool,
    pub whitelisted: bool,
    pub command: fn(Vec<&str>) -> Result<CommandResult, Error>,
}

impl Default for Command {
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

impl Command {
    pub fn execute(&self, args: Vec<&str>) -> Result<CommandResult, Error> {
        trace!("Executing command {} with args {:?}", self.name, args);
        (self.command)(args)
    }
}

fn noop(_args: Vec<&str>) -> Result<CommandResult, Error> {
    unimplemented!()
}

#[derive(Debug)]
pub struct Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Could not execute command")
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "A command could not be executed"
    }
}

pub struct CommandResult {
    pub message: Option<String>,
    pub arguments: Option<Vec<String>>,
}

impl Default for CommandResult {
    fn default() -> Self {
        Self {
            message: None,
            arguments: None,
        }
    }
}
