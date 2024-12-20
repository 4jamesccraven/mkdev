use std::fs;
use std::env;
use std::io;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};

pub fn make_relative(path: PathBuf) -> PathBuf {
    // Safe becuase appropriate checks will
    // have already been made
    let cwd = env::current_dir().unwrap();

    if path.starts_with(&cwd) {
        path.strip_prefix(cwd).unwrap_or(&path).to_path_buf()
    }
    else {
        path
    }
}

pub trait Displayable {
    fn display(&self, prefix: &str, last: bool);
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Content {
    File(File),
    Directory(Directory),
}

impl Displayable for Content {
    fn display(&self, prefix: &str, last: bool) {
        match self {
            Self::File(f) => f.display(prefix, last),
            Self::Directory(d) => d.display(prefix, last),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub name: String,
    pub content: String,
}

impl File {
    pub fn new(name: &str) -> Option<Self> {
        let content = fs::read_to_string(&name).ok()?;
        let name = name.to_string();

        Some(Self{
            name,
            content
        })
    }
}

impl Displayable for File {
    fn display(&self, prefix: &str, last: bool) {
        let line = if last { "└── " } else { "├── " };
        println!("{}{}{}", prefix, line, self.name);
    }
}

#[derive(Serialize, Deserialize, Debug)]
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
            }
            else if path.is_dir() {
                if let Ok(dir) = Directory::new(path.to_str().unwrap()) {
                    files.push(Content::Directory(dir));
                }
            }
        }

        let name = name.to_string();

        Ok(Self{
            name,
            files,
        })
    }
}

impl Displayable for Directory {
    fn display(&self, prefix: &str, last: bool) {
        let line = if last { "└── " } else { "├── " };
        println!("{}{}{}", prefix, line, self.name);

        let new_prefix = if last { "    " } else { "│   " };
        let new_prefix = format!("{prefix}{new_prefix}");

        let mut iter = self.files.iter().peekable();

        while let Some(obj) = iter.next() {
            obj.display(&new_prefix, iter.peek().is_none());
        }
    }
}

