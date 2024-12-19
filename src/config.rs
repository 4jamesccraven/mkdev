use std::fs;
use serde::{Serialize, Deserialize};
use dirs::data_dir;
use toml;

#[derive(Serialize, Deserialize, Debug)]
pub struct Recipe {
    pub name: String,
    pub description: String,
    pub languages: Vec<String>,
    pub contents: Vec<Content>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Content {
    File(File),
    Directory(Directory),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub name: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Directory {
    pub name: String,
    pub filies: Vec<File>,
}

pub fn gather() -> Result<Vec<Recipe>, &'static str> {
    let data_dir = data_dir()
        .ok_or("Fatal: unable to open acquire application data")?;

    let files = fs::read_dir(data_dir)
        .map_err(|_| "Fatal: unable to read application data")?;

    let mut recipes = Vec::new();

    for file in files {
        let file = file.map_err(|_| "Error reading file")?;
        let path = file.path();

        if path.extension() == Some(std::ffi::OsStr::new("toml")) && path.is_file() {
            let file_contents = fs::read_to_string(path)
                .map_err(|_| "Error Reading File")?;
            let recipe = toml::from_str(&file_contents);

            if let Ok(recipe) = recipe {
                recipes.push(recipe);
            }
        }
    }

    Ok(recipes)
}

