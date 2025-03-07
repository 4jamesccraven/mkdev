# Mkdev Configuration Wiki
Mkdev is configured by a TOML file stored at `$XDG_CONFIG_HOME/mkdev/config.toml`.
At the moment, there are only two tables, counting the global table. This may change
in the future.

## Global Options
- recipe_dir: path [optional]
    - The path to the directory where recipes should be stored.
    - Default: `empty` (evaluates to `"/home/user/.local/share/mkdev"`)
    - Example: `recipe_dir = "/home/user/Documents/mkdev_recipes"`

## Substitutions
This table contains any number of Substitutions that should be made when building a recipe.

- TEXT_TO_SUBSTITUE: string
    - The source, destination pair
    - Example: `pc_name = "hostname --fqdn"`

### Reserved Values
Some replacement values are provided by the program as they may be difficult to calculate using
other programs. Such values are prepended with `mk::`

- "mk::dir": The directory in which the recipe is being built.
