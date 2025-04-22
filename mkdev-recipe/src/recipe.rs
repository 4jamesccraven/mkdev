use crate::config::Config;
use crate::content::{Content, Directory};
use crate::subs::Replacer;

use std::collections::HashMap;
use std::env;
use std::fmt::Display;
use std::fs;
use std::io;
use std::path::PathBuf;

use dirs::data_dir;
use hyperpolyglot::{get_language_breakdown, Language};
use serde::{Deserialize, Serialize};
use toml;

/// Get the user's preferred data dir, or use the default XDG_DATA_DIR
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

        // Converts HashMap<&name, detected_info> -> Vec<(name, num_matching_files)>
        let mut breakdown: Vec<_> = get_language_breakdown(curr_dir)
            .iter()
            .map(|(lang, det)| (*lang, det.len()))
            .collect();

        // Sort languages by number of matching files
        breakdown.sort_by(|a, b| b.1.cmp(&a.1));

        let languages: Vec<_> = breakdown
            .iter()
            // Discard the count, as we only needed it to sort
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

    /// Attempt to find all user defined recipes, returning error values to propagate to
    /// error-handling functions
    pub fn gather() -> io::Result<HashMap<String, Recipe>> {
        let data_dir = get_data_dir()?;

        let files = fs::read_dir(data_dir)?;

        let mut recipes: Vec<Recipe> = Vec::new();

        for file in files {
            let path = file?.path();

            if path.extension() == Some(std::ffi::OsStr::new("toml")) && path.is_file() {
                let file_contents = fs::read_to_string(&path)?;
                let recipe: Result<Recipe, _> = toml::from_str(&file_contents);

                match recipe {
                    Ok(recipe) => {
                        recipes.push(recipe);
                    }
                    Err(_) => {
                        eprintln!("mkdev: warning: {} is not a valid recipe.", path.display());
                    }
                }
            }
        }

        let recipes = recipes
            .iter()
            .map(|r| (r.name.clone(), r.to_owned()))
            .collect();

        Ok(recipes)
    }

    /// Save the recipe object by serialising self into the data directory
    pub fn save(&self) -> io::Result<PathBuf> {
        let mut data_dir = get_data_dir()?;

        data_dir.push(format!("{}.toml", self.name));

        fs::write(&data_dir, toml::to_string_pretty(&self).unwrap())?;

        Ok(data_dir)
    }

    /// Delete the recipe by deleting its serialised self
    pub fn delete(&self) -> io::Result<PathBuf> {
        let mut data_dir = get_data_dir()?;

        data_dir.push(format!("{}.toml", self.name));

        fs::remove_file(&data_dir)?;

        Ok(data_dir)
    }

    /// Display contents of `tree` with default style
    pub fn display_contents(&self) -> String {
        let mut out = format!("\x1b[1m{}\x1b[0m\n", self.name);
        let mut iter = self.contents.iter().peekable();

        while let Some(obj) = iter.next() {
            let next = obj.produce_tree_string("".into(), iter.peek().is_none());
            out.push_str(&next);
        }

        out
    }

    /// Display all file names associated with the recipe
    pub fn display_contents_plain(&self) -> String {
        let mut out = String::new();

        for obj in &self.contents {
            match obj {
                Content::File(file) => {
                    let filname = format!("{}\n", file.name);
                    out.push_str(&filname);
                }
                Content::Directory(dir) => {
                    let dir_contents = format!("{}", dir.produce_file_names());
                    out.push_str(&dir_contents);
                }
            }
        }

        out
    }

    /// Build an individual recipe, recursing into sub-directories if there are any
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

impl Display for Recipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\x1b[1m{}\x1b[0m ( {} )\n  {}",
            self.name,
            self.languages.join(" "),
            self.description
        )
    }
}
