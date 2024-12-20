use std::fs;
use std::env;
use std::io;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use dirs::data_dir;
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

#[derive(Serialize, Deserialize, Debug)]
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
        let curr_dir = Directory::new(curr_dir.to_str().unwrap())?;

        let contents = curr_dir.files;
        let description = description.unwrap_or("".into());

        Ok(Self{
            name,
            contents,
            languages: Vec::new(),
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

        println!("Recipe saved successfully {}.", &data_dir.display());

        Ok(())
    }

    pub fn delete(&self) -> io::Result<()> {
        let mut data_dir = get_data_dir()?;

        data_dir.push(format!("{}.toml", self.name));

        fs::remove_file(&data_dir)?;

        println!("Deleted recipe at {}.", &data_dir.display());

        Ok(())
    }
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

