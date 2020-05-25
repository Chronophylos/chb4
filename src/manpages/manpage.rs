use super::ChapterName;
use snafu::{ResultExt, Snafu};
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
    about: String,
    description: String,
    example: Option<String>,
    characteristics: Vec<(String, String)>,
}

impl Manpage {
    pub fn new(
        names: Vec<String>,
        chapter: ChapterName,
        about: String,
        description: String,
        example: Option<String>,
        characteristics: Vec<(String, String)>,
    ) -> Self {
        Self {
            names,
            chapter,
            about,
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

    pub fn short(&self) -> String {
        format!(
            "{} - {} https://chb4.chronophylos.com/{}/{}",
            self.names.join(", "),
            self.about,
            self.chapter,
            self.name()
        )
    }

    fn render_title(&self) -> Result<String> {
        Ok(format!(
            "= {}
:icons: font
    Version: CHB4 {{pkg_version}} ({{git_hash}})",
            self
        ))
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
            self.about
        )
    }

    fn render_characteristics(&self) -> String {
        if self.characteristics.is_empty() {
            return String::from("");
        }

        let characterisitics: Vec<String> = self
            .characteristics
            .iter()
            .map(|t| format!("pass:normal[{}] {}", t.1, t.0))
            .collect();

        format!(
            "== CHARACTERISTICS

{}",
            characterisitics.join("\n\n")
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
        let path = path.as_ref();

        debug!("Writing {} to {}", self, path.display());

        let mut f = File::create(path).context(CreateFile)?;

        f.write_all(self.render()?.as_bytes()).context(WriteFile)
    }
}

impl fmt::Display for Manpage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name(), self.chapter)
    }
}
