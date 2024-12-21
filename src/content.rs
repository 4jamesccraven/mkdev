use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub fn make_relative(path: PathBuf) -> PathBuf {
    // Safe becuase appropriate checks will
    // have already been made
    let cwd = env::current_dir().unwrap();

    if path.starts_with(&cwd) {
        path.strip_prefix(cwd).unwrap_or(&path).to_path_buf()
    } else {
        path
    }
}

pub trait Displayable {
    fn display(&self, prefix: &str, last: bool);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Content {
    File(File),
    Directory(Directory),
}

impl Content {
    pub fn get_name(&self) -> String {
        match self {
            Content::File(file) => file.name.clone(),
            Content::Directory(dir) => dir.name.clone(),
        }
    }
}

impl Displayable for Content {
    fn display(&self, prefix: &str, last: bool) {
        match self {
            Self::File(f) => f.display(prefix, last),
            Self::Directory(d) => d.display(prefix, last),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct File {
    pub name: String,
    pub content: String,
}

impl File {
    pub fn new(name: &str) -> Option<Self> {
        let content = fs::read_to_string(&name).ok()?;
        let name = name.to_string();

        Some(Self { name, content })
    }
}

impl Displayable for File {
    fn display(&self, prefix: &str, last: bool) {
        let line = if last { "└── " } else { "├── " };

        let text = if let Some(pos) = self.name.rfind('/') {
            let (_, file) = self.name.split_at(pos + 1);
            format!("{}", file)
        } else {
            format!("{}", self.name)
        };

        println!("\x1b[38;5;8m{}{}\x1b[0m{}", prefix, line, text);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Directory {
    pub name: String,
    pub files: Vec<Content>,
}

impl Directory {
    pub fn new(name: &str) -> io::Result<Self> {
        let file_iter = fs::read_dir(&name)?;

        let mut files = Vec::new();

        for file in file_iter {
            let path = file?.path();
            let path = make_relative(path);

            if let None = path.to_str() {
                continue;
            }
            let path_str = path.to_str().unwrap();

            if path.is_file() {
                if let Some(file) = File::new(path_str) {
                    files.push(Content::File(file));
                }
            } else if path.is_dir() {
                if let Ok(dir) = Directory::new(path.to_str().unwrap()) {
                    files.push(Content::Directory(dir));
                }
            }
        }

        let name = name.to_string();

        Ok(Self { name, files })
    }

    pub fn sort(&mut self) {
        self.files.sort_by(|a, b| match (a, b) {
            (Content::Directory(dir_a), Content::Directory(dir_b)) => dir_a.name.cmp(&dir_b.name),
            (Content::Directory(_), Content::File(_)) => std::cmp::Ordering::Less,
            (Content::File(_), Content::Directory(_)) => std::cmp::Ordering::Greater,
            (Content::File(file_a), Content::File(file_b)) => file_a.name.cmp(&file_b.name),
        });

        for content in self.files.iter_mut() {
            if let Content::Directory(dir) = content {
                dir.sort()
            }
        }
    }
}

impl Displayable for Directory {
    fn display(&self, prefix: &str, last: bool) {
        let line = if last { "└── " } else { "├── " };

        let text = format!("\x1b[34m{}\x1b[0m", self.name);

        println!("\x1b[38;5;8m{}{}\x1b[0m{}", prefix, line, text);

        let new_prefix = if last { "    " } else { "│   " };
        let new_prefix = format!("{prefix}{new_prefix}");

        let mut iter = self.files.iter().peekable();

        while let Some(obj) = iter.next() {
            obj.display(&new_prefix, iter.peek().is_none());
        }
    }
}
