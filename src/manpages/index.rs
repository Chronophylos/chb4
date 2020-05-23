use super::{Chapter, ChapterName, Manpage, ManpageProducer};
use snafu::{ResultExt, Snafu};
use std::{collections::HashMap, fs, io, path::Path, sync::Arc};

static FILE_EXTENSION: &str = "adoc";

#[derive(Debug, Snafu)]
pub enum Error {
    NotADirectory,
    RenderPage { source: super::manpage::Error },
    CreateDir { source: io::Error },
    CanonicalizePath { source: io::Error },
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Default, Debug)]
pub struct Index {
    chapters: HashMap<ChapterName, Chapter>,
}

impl Index {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn populate<T>(&mut self, pages: Vec<Arc<T>>)
    where
        T: ManpageProducer,
    {
        for page in pages.iter().map(|p| Arc::new(p.get_manpage())) {
            match self.chapters.get_mut(&page.chapter) {
                Some(pages) => pages.insert(page),
                None => {
                    self.chapters
                        .insert(page.chapter.clone(), Chapter::with_page(page));
                }
            }
        }
    }

    pub fn whatis(&self, chapter: Option<ChapterName>, name: String) -> Option<Arc<Manpage>> {
        match chapter {
            Some(c) => match self.chapters.get(&c) {
                Some(c) => c.get_page(name),
                None => None,
            },
            None => self
                .chapters
                .iter()
                .find_map(|(_, c)| c.get_page(name.clone())),
        }
    }

    pub fn page_count(&self) -> usize {
        self.chapters
            .values()
            .fold(0, |acc, x| acc + x.page_count())
    }

    pub fn render_toc(&self) -> String {
        unimplemented!()
    }

    pub fn write<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        if !path.exists() {
            fs::create_dir_all(&path).context(CreateDir)?;
        } else if !path.is_dir() {
            return Err(Error::NotADirectory);
        }

        let path = fs::canonicalize(path).context(CanonicalizePath)?;

        debug!("Writing documentation to {}", path.display());

        for (chapter_name, chapter) in self.chapters.iter() {
            let path = path.join(chapter_name.to_string());

            if !path.exists() {
                fs::create_dir_all(&path).context(CreateDir)?;
            } else if !path.is_dir() {
                return Err(Error::NotADirectory);
            }

            chapter
                .page_iter()
                .map(|page| {
                    page.render_file(path.join(format!("{}.{}", page.name(), FILE_EXTENSION)))
                        .context(RenderPage)
                })
                .collect::<Result<_>>()?;
        }

        Ok(())
    }
}
