use snafu::{OptionExt, ResultExt, Snafu};
use std::{fs::File, io::prelude::*, path::Path};

#[derive(Debug, Snafu)]
pub enum Error {
    GetName,
    CreateFile { source: std::io::Error },
    WriteFile { source: std::io::Error },
}

type Result<T> = std::result::Result<T, Error>;

pub trait Manpage {
    fn names(&self) -> &Vec<String>;
    fn chapter(&self) -> u8;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn example(&self) -> Option<&str>;
    fn characteristics(&self) -> &Vec<(String, String)>;
}

impl dyn Manpage {
    fn render_title(&self) -> Result<String> {
        Ok(format!("= {}", self.names().get(0).context(GetName)?))
    }

    fn render_aliases(&self) -> String {
        if self.names().len() < 2 {
            String::from("")
        } else {
            format!(
                "Aliases: {:?}",
                self.names().iter().skip(1).collect::<Vec<_>>()
            )
        }
    }

    fn render_name(&self) -> String {
        format!(
            "== NAME

{}",
            self.name()
        )
    }

    fn render_characteristics(&self) -> String {
        if self.characteristics().is_empty() {
            return String::from("");
        }

        let characterisitics: Vec<String> = self
            .characteristics()
            .iter()
            .map(|t| format!("| {}\n| {}", t.0, t.1))
            .collect();

        format!(
            "== CHARACTERISTICS

|===
{}
|===",
            characterisitics.join("\n")
        )
    }

    fn render_description(&self) -> String {
        format!(
            "== DESCRIPTION

{}",
            self.description()
        )
    }

    pub fn render(&self) -> Result<String> {
        let chunks = vec![
            self.render_title()?,
            self.render_aliases(),
            self.render_name(),
            self.render_characteristics(),
            self.render_description(),
        ];

        Ok(chunks.join("\n\n"))
    }

    pub fn render_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut f = File::create(path).context(CreateFile)?;

        f.write_all(self.render()?.as_bytes()).context(WriteFile)
    }
}
