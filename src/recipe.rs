use std::fs;
use std::env;
use std::io;
use std::path::PathBuf;

use crate::content::{Content, Directory, Displayable};

use dirs::data_dir;
use hyperpolyglot::{Language, get_language_breakdown};
use serde::{Serialize, Deserialize};
use toml;

pub fn get_data_dir() -> io::Result<PathBuf> {
    let err = io::Error::new(io::ErrorKind::Other, "Error getting data directory");
    let mut data_dir = data_dir().ok_or(err)?;
    data_dir.push("mkdev");

    if !data_dir.is_dir() {
        fs::create_dir_all(&data_dir)?;
    }

    Ok(data_dir)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Recipe {
    pub name: String,
    pub description: String,
    pub languages: Vec<String>,
    pub contents: Vec<Content>,
}

impl Recipe {
    pub fn imprint(name: String, description: Option<String>) -> io::Result<Self> {
        let curr_dir = env::current_dir()?;

        if let None = curr_dir.to_str() {
            return Err(
                io::Error::new(io::ErrorKind::Other, "Error reading file")
            );
        }
        let mut dir_obj = Directory::new(curr_dir.to_str().unwrap())?;
        dir_obj.sort();

        let contents = dir_obj.files;
        let description = description.unwrap_or("".into());

        let mut breakdown: Vec<_> = get_language_breakdown(curr_dir)
            .iter()
            .map(|(lang, det)| {
                (*lang, det.len())
            })
            .collect();

        breakdown.sort_by(|a, b| {
            b.1.cmp(&a.1)
        });

        let languages: Vec<_> = breakdown
            .iter()
            .filter_map(|(lang, _)| Language::try_from(*lang).ok())
            .map(|lang| {
               if let Some(hex) = lang.color {
                   let hex = &hex[1..].to_string();

                   let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
                   let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
                   let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);

                   format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, lang.name)
               } else {
                   format!("\x1b[37m{}\x1b[0m", lang.name)
               }
            })
            .collect();

        Ok(Self{
            name,
            contents,
            languages,
            description,
        })
    }

    pub fn gather() -> io::Result<Vec<Recipe>> {
        let data_dir = get_data_dir()?;

        let files = fs::read_dir(data_dir)?;

        let mut recipes = Vec::new();

        for file in files {
            let path = file?.path();

            if path.extension() == Some(std::ffi::OsStr::new("toml")) && path.is_file() {
                let file_contents = fs::read_to_string(path)?;
                let recipe = toml::from_str(&file_contents);

                if let Ok(recipe) = recipe {
                    recipes.push(recipe);
                }
            }
        }

        Ok(recipes)
    }

    pub fn save(&self) -> io::Result<()> {
        let mut data_dir = get_data_dir()?;

        data_dir.push(format!("{}.toml", self.name));

        fs::write(&data_dir, toml::to_string(&self).unwrap())?;

        println!("Recipe saved successfully to {}.", &data_dir.display());

        Ok(())
    }

    pub fn delete(&self) -> io::Result<()> {
        let mut data_dir = get_data_dir()?;

        data_dir.push(format!("{}.toml", self.name));

        fs::remove_file(&data_dir)?;

        println!("Deleted recipe at {}.", &data_dir.display());

        Ok(())
    }

    pub fn list(&self, tree: bool) {
        println!("\x1b[1m{}\x1b[0m", self.name);
        if tree {
            let mut iter = self.contents.iter().peekable();

            while let Some(obj) = iter.next() {
                obj.display("".into(), iter.peek().is_none());
            }
        } else {
            print!("  ");
            for language in &self.languages {
                print!("{} ", language);
            }
            println!()
        }
    }

    pub fn build(dir: &PathBuf, contents: &Vec<Content>, verbose: bool) -> io::Result<()> {
        for content in contents {
            let mut path = dir.clone();
            path.push(&content.get_name());

            if verbose {
                println!("{}", path.display());
            }

            match content {
                Content::File(file) => {
                    fs::write(&path, &file.content)?;
                },
                Content::Directory(directory) => {
                    fs::create_dir_all(&path)?;
                    Recipe::build(&dir, &directory.files, verbose)?;
                },
            }
        }

        Ok(())
    }
}
