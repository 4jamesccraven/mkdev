use std::collections::BTreeMap;
use std::path::Path;

use super::*;

use colored::Colorize;

const WIRE: &str = "│   ";
const CONNECTOR: &str = "├── ";
const CAP: &str = "└── ";
const GAP: &str = "    ";

/// Create a String that represents the file system akin to the output of the program "tree"
pub fn repr_tree(files: &[RecipeItem]) -> String {
    let tree = build_recursive_content(files);
    make_tree_string(&tree, "".into())
}

fn build_recursive_content(files: &[RecipeItem]) -> Vec<TreeContent> {
    use RecipeItem::*;
    // Create an intermediate tree
    let mut root = TreeNode::new();

    // Insert each file into the tree
    for file in files {
        match file {
            File(file) => {
                root.insert(file.name.as_path(), true);
            }
            Directory(name) => {
                root.insert(name, false);
            }
        }
    }

    // Convert sub-components of the tree to a more fully-featured recursive representation
    let mut out: Vec<_> = root
        .children
        .into_iter()
        .map(|(name, node)| node.into_tree_content(name))
        .collect();

    out.sort_unstable();
    out
}

fn make_tree_string(cont: &[TreeContent], prefix: String) -> String {
    let mut out = String::new();
    let mut rec_iter = cont.iter().peekable();

    while let Some(file) = rec_iter.next() {
        use TreeContent::*;

        // If this is the last item we need to cap off our prefix with a symbol that shows it,
        // otherwise we just continue connecting it with previous lines
        let is_last = rec_iter.peek().is_none();
        let line = if is_last { CAP } else { CONNECTOR };

        match file {
            // Trivial case, just display the whole prefix and the file
            Leaf { name, empty_dir } => {
                #[rustfmt::skip]
                let name = if *empty_dir { name.blue() } else { name.normal()
                };
                let new_line = format!(
                    "{}{}{}\n",
                    prefix.truecolor(128, 128, 128),
                    line.truecolor(128, 128, 128),
                    name
                );
                out.push_str(&new_line);
            }
            HasChildren { name, contents } => {
                // Display directory's name
                let new_line = format!(
                    "{}{}{}\n",
                    prefix.truecolor(128, 128, 128),
                    line.truecolor(128, 128, 128),
                    name.blue()
                );

                // Pad the new prefix with indentation if this directory is the last item, of the
                // current iteration otherwise use a line that continues down and connects the
                // structure
                let new_prefix = format!("{}{}", prefix, if is_last { GAP } else { WIRE });

                // Recurse through this sub-directory
                let rec = make_tree_string(contents, new_prefix);

                out.push_str(&new_line);
                out.push_str(&rec);
            }
        }
    }

    out
}

/// Internal Data Type to represent a recursive file tree (akin to previous versions of mkdev)
enum TreeContent {
    Leaf {
        name: String,
        empty_dir: bool,
    },
    HasChildren {
        name: String,
        contents: Vec<TreeContent>,
    },
}

/// Intermediate representation of the file tree
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
    is_file: bool,
}

impl TreeNode {
    fn new() -> Self {
        TreeNode {
            children: BTreeMap::new(),
            is_file: false,
        }
    }

    fn insert(&mut self, path: &Path, is_file: bool) {
        let mut current = self;
        // Break the path into its components, and at them one-by-one to the tree
        for comp in path.components() {
            let name = comp.as_os_str().to_string_lossy().into_owned();
            current = current.children.entry(name).or_insert_with(TreeNode::new)
        }
        // Once we have reached the end of the recursion, mark it as a file if it is one.
        if is_file {
            current.is_file = true;
        }
    }

    fn into_tree_content(self, name: String) -> TreeContent {
        use TreeContent::*;
        if self.children.is_empty() {
            Leaf {
                name,
                empty_dir: !self.is_file,
            }
        } else {
            let mut contents: Vec<_> = self
                .children
                .into_iter()
                .map(|(name, node)| node.into_tree_content(name))
                .collect();

            contents.sort_unstable();

            HasChildren {
                name,
                // Represent the child nodes
                contents,
            }
        }
    }
}

impl PartialEq for TreeContent {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl Eq for TreeContent {}

impl PartialOrd for TreeContent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TreeContent {
    fn cmp(&self, other: &Self) -> Ordering {
        use std::cmp::Ordering::*;
        use TreeContent::*;
        // In all cases, this places a directory earlier (less) than a file, and later (greater)
        // for a file. If two of the same type are encountered, they are sorted lexographically as
        // a fallback
        match (self, other) {
            // Two leaves
            (
                Leaf { name, empty_dir },
                Leaf {
                    name: other_name,
                    empty_dir: other_dir,
                },
            ) => match (empty_dir, other_dir) {
                // Directories first
                (true, false) => Less,
                (false, true) => Greater,
                // Fallback to name
                _ => name.cmp(other_name),
            },
            // Mixed cases
            (
                Leaf { name, empty_dir },
                HasChildren {
                    name: other_name,
                    contents: _,
                },
            ) => {
                // If it's not a directory, then later
                if !empty_dir {
                    Greater
                // If it is, fallback to names
                } else {
                    name.cmp(other_name)
                }
            }
            (
                HasChildren { name, contents: _ },
                Leaf {
                    name: other_name,
                    empty_dir,
                },
            ) => {
                // If the other isn't also a directory, earlier
                if !empty_dir {
                    Less
                // If it is, fallback to names
                } else {
                    name.cmp(other_name)
                }
            }
            // Two directories
            // just compare names
            (
                HasChildren { name, contents: _ },
                HasChildren {
                    name: other_name,
                    contents: _,
                },
            ) => name.cmp(other_name),
        }
    }
}
