use crate::manpages::Manpage;
use std::{
    collections::{hash_map::Values, HashMap},
    fmt,
    sync::Arc,
};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum ChapterName {
    Action,
    Command,
    Unkown,
}

impl fmt::Display for ChapterName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Action => "action",
                Self::Command => "command",
                Self::Unkown => "unknown",
            }
        )
    }
}

impl From<String> for ChapterName {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "action" | "1" => Self::Action,
            "command" | "2" => Self::Command,
            _ => Self::Unkown,
        }
    }
}

#[derive(Default, Debug)]
pub struct Chapter {
    pages: HashMap<String, Arc<Manpage>>,
    aliases: HashMap<String, String>,
}

impl Chapter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_page(page: Arc<Manpage>) -> Self {
        let mut chapter = Chapter::new();
        chapter.insert(page);
        chapter
    }

    pub fn insert(&mut self, page: Arc<Manpage>) {
        let name = page.name().to_string();
        for alias in page.other_names() {
            self.aliases.insert(alias, name.clone());
        }
        self.pages.insert(name, page);
    }

    pub fn get_page(&self, name: String) -> Option<Arc<Manpage>> {
        let name = self.aliases.get(&name).unwrap_or_else(|| &name);
        self.pages.get(name).cloned()
    }

    pub fn page_iter(&self) -> Values<String, Arc<Manpage>> {
        self.pages.values()
    }

    pub fn page_count(&self) -> usize {
        self.pages.values().count()
    }
}
