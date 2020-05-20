use super::ChapterName;
use snafu::{OptionExt, ResultExt, Snafu};
use std::{fmt, fs::File, io::prelude::*, path::Path};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Getting name from names()"))]
    GetName,

    #[snafu(display("Creating file: {}", source))]
    CreateFile { source: std::io::Error },

    #[snafu(display("Writing file: {}", source))]
    WriteFile { source: std::io::Error },
}

type Result<T> = std::result::Result<T, Error>;

pub trait ManpageProducer {
    fn get_manpage(&self) -> Manpage;
}

#[derive(Debug)]
pub struct Manpage {
    names: Vec<String>,
    pub chapter: ChapterName,
    name: String,
    description: String,
    example: Option<String>,
    characteristics: Vec<(String, String)>,
}

impl Manpage {
    pub fn new(
        names: Vec<String>,
        chapter: ChapterName,
        name: String,
        description: String,
        example: Option<String>,
        characteristics: Vec<(String, String)>,
    ) -> Self {
        Self {
            names,
            chapter,
            name,
            description,
            example,
            characteristics,
        }
    }

    pub fn name(&self) -> &str {
        self.names.get(0).unwrap()
    }

    pub fn other_names(&self) -> Vec<String> {
        self.names[1..].to_vec()
    }

    fn render_title(&self) -> Result<String> {
        Ok(format!("= {}", self.names.get(0).context(GetName)?))
    }

    fn render_aliases(&self) -> String {
        if self.names.len() < 2 {
            String::from("")
        } else {
            format!(
                "Aliases: {:?}",
                self.names.iter().skip(1).collect::<Vec<_>>()
            )
        }
    }

    fn render_name(&self) -> String {
        format!(
            "== NAME

{}",
            self.name
        )
    }

    fn render_characteristics(&self) -> String {
        if self.characteristics.is_empty() {
            return String::from("");
        }

        let characterisitics: Vec<String> = self
            .characteristics
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
            self.description
        )
    }

    fn render_example(&self) -> String {
        match self.example.clone() {
            Some(s) => format!(
                "== EXAMPLE

{}",
                s
            ),
            None => String::new(),
        }
    }

    pub fn render(&self) -> Result<String> {
        let chunks = vec![
            self.render_title()?,
            self.render_aliases(),
            self.render_name(),
            self.render_characteristics(),
            self.render_description(),
            self.render_example(),
        ];

        Ok(chunks.join("\n\n"))
    }

    pub fn render_file<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let mut f = File::create(path).context(CreateFile)?;

        f.write_all(self.render()?.as_bytes()).context(WriteFile)
    }
}

impl fmt::Display for Manpage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())?;
        if self.names.len() > 1 {
            write!(f, " ({})", self.other_names().join(", "))?;
        }
        write!(f, " {}", self.name)
    }
}
