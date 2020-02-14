use super::command::{Command, CommandResult, Error};

pub fn command() -> Command {
    fn command(args: Vec<&str>) -> Result<CommandResult, Error> {
        if args.is_empty() {
            return Ok(CommandResult {
                message: Some(String::from("Test what?")),
                ..Default::default()
            });
        }

        Ok(CommandResult {
            message: Some(String::from("Testing ") + args[0]),
            ..Default::default()
        })
    }

    Command {
        name: String::from("test"),
        command,
        ..Default::default()
    }
}
