use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

fn make_relative(path: PathBuf) -> PathBuf {
    let cwd = env::current_dir().expect("Appropriate checks will have been made by this point.");

    if path.starts_with(&cwd) {
        path.strip_prefix(cwd).unwrap_or(&path).to_path_buf()
    } else {
        path
    }
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

    pub fn produce_tree_string(&self, prefix: &str, last: bool) -> String {
        match self {
            Self::File(f) => f.produce_tree_string(prefix, last),
            Self::Directory(d) => d.produce_tree_string(prefix, last),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct File {
    pub name: String,
    pub content: String,
}

impl File {
    pub fn new(name: &str) -> io::Result<Self> {
        let content = fs::read_to_string(&name)?;
        let name = name.to_string();

        Ok(Self { name, content })
    }

    pub fn produce_tree_string(&self, prefix: &str, last: bool) -> String {
        let line = if last { "└── " } else { "├── " };

        let text = if let Some(pos) = self.name.rfind('/') {
            let (_, file) = self.name.split_at(pos + 1);
            file.to_string()
        } else {
            self.name.clone()
        };

        format!("\x1b[38;5;8m{}{}\x1b[0m{}\n", prefix, line, text)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Directory {
    pub name: String,
    pub files: Vec<Content>,
}

impl Directory {
    pub fn new(name: &str) -> io::Result<Self> {
        let file_iter = fs::read_dir(name)?;

        let mut files = Vec::new();

        for entry in file_iter {
            let path = entry?.path();
            let path = make_relative(path);

            let path_str = path.to_str().map_or_else(
                // If path is not valid UTF-8, try a lossy conversion
                || path.to_string_lossy().into_owned(),
                String::from,
            );

            if path.is_file() {
                let file = File::new(&path_str)?;
                files.push(Content::File(file));
            } else if path.is_dir() {
                let dir = Directory::new(&path_str)?;
                files.push(Content::Directory(dir));
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

    pub fn produce_tree_string(&self, prefix: &str, last: bool) -> String {
        let line = if last { "└── " } else { "├── " };

        let text = format!("\x1b[34m{}\x1b[0m", self.name);

        let mut out = format!("\x1b[38;5;8m{}{}\x1b[0m{}\n", prefix, line, text);

        let new_prefix = if last { "    " } else { "│   " };
        let new_prefix = format!("{prefix}{new_prefix}");

        let mut iter = self.files.iter().peekable();

        while let Some(obj) = iter.next() {
            let next = obj.produce_tree_string(&new_prefix, iter.peek().is_none());
            out.push_str(&next);
        }

        out
    }

    pub fn produce_file_names(&self) -> String {
        let mut out = format!("{}/\n", self.name);

        for file in &self.files {
            match file {
                Content::File(file) => {
                    let file_name = format!("{}\n", file.name);
                    out.push_str(&file_name);
                }
                Content::Directory(directory) => {
                    let contents = directory.produce_file_names();
                    out.push_str(&contents);
                }
            }
        }

        out
    }
}
