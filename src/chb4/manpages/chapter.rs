use std::fmt;

#[derive(PartialEq, Eq, Hash)]
pub enum Chapter {
    Action,
    Command,
}

impl fmt::Display for Chapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Action => "action",
                Self::Command => "command",
            }
        )
    }
}
