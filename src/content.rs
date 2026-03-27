//! Utilties for constructing and manipulating the contents of a mkdev recipe.
use ignore::overrides::OverrideBuilder;

use crate::cli::Imprint;
use crate::ctx;
use crate::mkdev_error;

use std::cmp::Ordering;
use std::fs;
use std::io;
use std::path::PathBuf;

use ignore::{Walk, WalkBuilder};
use serde::{Deserialize, Serialize};

/// The data a mkdev recipe stores.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RecipeItem {
    File(File),
    Directory(PathBuf),
}

impl RecipeItem {
    /// Returns the name of the `RecipeItem`.
    pub fn name(&self) -> String {
        let name = match self {
            RecipeItem::File(file) => file.name.to_string_lossy(),
            RecipeItem::Directory(dir) => dir.to_string_lossy(),
        };

        name.into()
    }

    /// Constructs a new `RecipeItem::File` variant
    fn file(name: PathBuf) -> io::Result<Self> {
        let f = File::new(name)?;
        Ok(Self::File(f))
    }

    /// Constructs a new `RecipeItem::Directory` variant
    fn dir(name: PathBuf) -> Self {
        Self::Directory(name)
    }
}

/// A file.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct File {
    pub name: PathBuf,
    pub content: String,
}

impl File {
    pub fn new(name: PathBuf) -> io::Result<Self> {
        let content = fs::read_to_string(&name)?;

        Ok(Self { name, content })
    }
}

/// Recursively detects and saves every file and subdirectory in the current working directory.
///
/// Standard ignore filters are applied (.gitignore, .ignore, etc.), and symlinks are ignored.
pub fn make_contents(walk: Walk) -> io::Result<Vec<RecipeItem>> {
    let cwd = std::env::current_dir()?;
    let mut out = vec![];

    for file in walk.flatten() {
        if file.path() == cwd {
            continue;
        }

        let data = file
            .file_type()
            .expect("only none for stdin, which is not allowed.");

        let mut path = file.into_path();

        if path.starts_with(&cwd) {
            path = path
                .strip_prefix(&cwd)
                .expect("prefix is confirmed to exist")
                .into();
        }

        let (is_file, is_dir, is_symlink) = (data.is_file(), data.is_dir(), data.is_symlink());

        // Make File or Directory variant as necessary
        match (is_file, is_dir, is_symlink) {
            (true, false, false) => out.push(RecipeItem::file(path)?),
            (false, true, false) => out.push(RecipeItem::dir(path)),
            // ignore symlinks (TODO: allow customisation with CLI)
            (false, false, true) => continue,
            // All of these methods' results are mutually exclusive
            // see: https://doc.rust-lang.org/nightly/std/fs/struct.FileType.html
            _ => unreachable!(),
        }
    }

    out.sort();
    Ok(out)
}

/// Constructs a recursive walk.
///
/// This function exists to allow users to override the walk behaviour.
pub fn build_walk(args: &Imprint) -> Result<Walk, mkdev_error::Error> {
    let cwd = ctx!(std::env::current_dir(), "getting cwd")?;

    let mut ob = OverrideBuilder::new(&cwd);
    for over in &args.exclude {
        ob.add(&format!("!{over}"))?;
    }
    let user_filters = ob.build()?;

    Ok(WalkBuilder::new(&cwd)
        .standard_filters(!args.no_filter)
        .overrides(user_filters)
        .build())
}

impl PartialEq for RecipeItem {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for RecipeItem {}

impl PartialOrd for RecipeItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RecipeItem {
    fn cmp(&self, other: &Self) -> Ordering {
        use RecipeItem::*;
        match (self, other) {
            (Directory(_), File(_)) => Ordering::Less,
            (File(_), Directory(_)) => Ordering::Greater,
            (File(a), File(b)) => a.name.cmp(&b.name),
            (Directory(a), Directory(b)) => a.cmp(b),
        }
    }
}
