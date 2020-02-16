mod test;
use super::command::Command;

pub fn test<'a>() -> Command<'a> {
    test::command()
}
