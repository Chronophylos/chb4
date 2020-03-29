use super::{Chapter, Manpage, ManpageTrait};
use snafu::{ResultExt, Snafu};
use std::{collections::HashMap, path::Path, sync::Arc};

static FILE_EXTENSION: &str = "adoc";

#[derive(Debug, Snafu)]
pub enum Error {
    NotADirectory,
    RenderPage { source: super::manpage::Error },
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Default, Clone)]
pub struct Index<'a> {
    pages: HashMap<Chapter, Vec<Arc<Manpage<'a>>>>,
}

impl<T> Index<'_, T>
where
    T: ManpageTrait,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn populate(&mut self, pages: Vec<Arc<T>>) {
        for page in pages {
            match self.pages.get_mut(&page.chapter()) {
                Some(pages) => pages.push(Manpage::new(page.clone())),
                None => {
                    self.pages
                        .insert(page.chapter(), vec![Manpage::new(page.clone())]);
                }
            };
        }
    }

    pub fn render_toc(&self) -> String {
        unimplemented!()
    }

    pub fn write<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        if !path.is_dir() {
            return Err(Error::NotADirectory);
        }

        info!("Writing documentation to {}", path.display());

        for (chapter, pages) in self.pages.iter() {
            let path = path.join(chapter.to_string());
            pages
                .iter()
                .map(|page| {
                    page.render_file(path.join(format!("{}.{}", page.name(), FILE_EXTENSION)))
                        .context(RenderPage)
                })
                .collect::<Result<_>>()?;
        }

        Ok(())
    }
}
