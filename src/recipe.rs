use crate::config::Config;
use crate::content::{Content, Directory, TreeDisplayItem};
use crate::subs::Replacer;

use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use dirs::data_dir;
use hyperpolyglot::{get_language_breakdown, Language};
use serde::{Deserialize, Serialize};
use toml;

pub fn get_data_dir() -> io::Result<PathBuf> {
    let err = io::Error::new(io::ErrorKind::Other, "Error getting data directory");
    let cfg = Config::get();

    let data_dir = match &cfg.recipe_dir {
        Some(dir) => dir.clone(),
        None => {
            let mut temp = data_dir().ok_or(err)?;
            temp.push("mkdev");
            temp
        }
    };

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
        let curr_dir: PathBuf = env::current_dir()?;

        let curr_dir_str = curr_dir
            .to_str()
            .map_or_else(|| curr_dir.to_string_lossy().into_owned(), String::from);

        let mut dir_obj = Directory::new(&curr_dir_str)?;
        dir_obj.sort();

        let contents = dir_obj.files;
        let description = description.unwrap_or("".into());

        let mut breakdown: Vec<_> = get_language_breakdown(curr_dir)
            .iter()
            .map(|(lang, det)| (*lang, det.len()))
            .collect();

        // Sort languages by percentage of recipe
        breakdown.sort_by(|a, b| b.1.cmp(&a.1));

        let languages: Vec<_> = breakdown
            .iter()
            .filter_map(|(lang, _)| Language::try_from(*lang).ok())
            .map(|lang| {
                if let Some(hex) = lang.color {
                    let hex = &hex[1..].to_string();

                    // Falls back to 255, 255, 255 when unable to correctly parse
                    // colour from hyperpolyglot
                    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
                    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
                    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);

                    format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, lang.name)
                } else {
                    format!("\x1b[37m{}\x1b[0m", lang.name)
                }
            })
            .collect();

        Ok(Self {
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
        if tree {
            println!("\x1b[1m{}\x1b[0m", self.name);
            let mut iter = self.contents.iter().peekable();

            while let Some(obj) = iter.next() {
                obj.display("".into(), iter.peek().is_none());
            }
        } else {
            print!("\x1b[1m{}\x1b[0m ( ", self.name);
            for language in &self.languages {
                print!("{} ", language);
            }
            println!(")");

            if !&self.description.is_empty() {
                println!("  {}", &self.description);
            }
        }
    }

    pub fn build(
        dir: &PathBuf,
        contents: &Vec<Content>,
        verbose: bool,
        re: &Replacer,
    ) -> io::Result<()> {
        if !dir.is_dir() {
            fs::create_dir_all(&dir)?;
        }

        for content in contents {
            let mut path = dir.clone();
            let name = re.sub(&content.get_name(), dir);
            path.push(name);

            if verbose {
                println!("{}", path.display());
            }

            match content {
                Content::File(file) => {
                    let content = re.sub(&file.content, dir);
                    fs::write(&path, content)?;
                }
                Content::Directory(directory) => {
                    fs::create_dir_all(&path)?;
                    Recipe::build(&dir, &directory.files, verbose, re)?;
                }
            }
        }

        Ok(())
    }
}
