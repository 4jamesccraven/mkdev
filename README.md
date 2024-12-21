```
                                           _____  _____ 
                                          |____ ||  _  |
 _______ _     _ ______  _______ _    _       / /| |/' |
 |  |  | |____/  |     \ |______  \  /        \ \|  /| |
 |  |  | |    \_ |_____/ |______   \/     .___/ /\ |_/ /
                                          \____(_)\___/ 
```
![image](https://img.shields.io/badge/release-3.0.0-orange)
![image](https://img.shields.io/badge/license-MIT_License-orange)

A CLI Tool for Tinkerers
------------------------
mkdev is a tool for easily generating boilerplate in programs,
scripts, and projects.

Features
--------
- Automated copying and pasting of directories and their contents
- Simple text substitutions for certain predefined values:
  - Destination Directory
  - Username
  - Date Info (day, month, year, weekday)
  - Automatic Github-style language detection for recipes
  - Tools to visualise recipe contents

History
------
Mkdev 3.0 is a spiritual successor to 2.0, which was written in python
and functioned far differently. I originally wrote mkdev because I wanted
something for simple scripting like Makefile without writing a new file for
every project. The script was hardcoded, which wasn't ideal for extensibility.
1.2 improved on the idea by making it so users could write configs that would
define the recipes, but it was clunky, requiring nested directories and poorly
conceived config structure. 2.0 improved on the structure of the config by
flattening the structure, but this made it difficult for a human to read,
necessitating a custom, buggy built-in tui text editor... very cool, but
definitely not ideal.

So the motivation of this re-write was two-fold: improve the ergononmics of the
program and user a language better suited and **faster**. (Also because [Steven](https://github.com/Steven-S1020)
kept bugging me about it /j)

- [mkdev 1.2](https://github.com/4jamesccraven/mkdev/tree/4d4ac6dd5fe044b7ba3d71d610716b5f3b9685d7)
- [mkdev 2.0](https://github.com/4jamesccraven/mkdev/tree/v2)
- *[what is that /j thing?](https://toneindicators.carrd.co/#introduction)*


Installation
------------
### Using Cargo
```
$ cargo install --git https://github.com/4jamesccraven/mkdev
```
### Using Nix
First ensure you have [flakes](https://wiki.nixos.org/wiki/Flakes) enabled.

`flake.nix`
```nix
{
  description = "mkdev installation example";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    mkdev = {
      url = "github:4jamesccraven/mkdev";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {nixpkgs, ...}@inputs:
    {
       nixosConfigurations.HOSTNAME = nixpkgs.lib.nixosSystem {
         specialArgs = {
           inherit inputs;
         };
         modules = [
           ./configuration.nix
         ];
       };
    };
}
```
later, where you keep your packages:
```nix
{pkgs, inputs}:

{
  environment.systemPackages = with pkgs; [
    # Other packages
    ...
    inputs.mkdev.packages.SYSTEM_ARCHITECTURE.mkdev
    ...
  ];
}
```
note that `SYSTEM_ARCHITECTURE` is a placeholder. In general it is
ARCHITECTURE-OS; e.g. `x86_64-linux` (most common, you probably have this
if on NixOS), `x86_64-darwin` (MacOS), `aarch64-linux`, etc.


Usage
-----
To get started, set up a directory as you normally would.
We'll use a simple python project as an example:
```
📂 mdev
├── 📄 flake.lock
├── 📄 flake.nix
├── 📄 main.py
└── 📄 requirements.txt
```
Then from the root of the directory we can copy it like so:
```
$ mk imprint test
Recipe saved successfully to /home/USER/.local/share/mkdev/test.toml.
```
Now the directory is saved and can be deployed anywhere! by running `mk test`
(because we saved it as test). 
Additionally, if we want to specify where it goes, this can be indicated with a `--`,
like so:
```
$ mk test -- PATH/TO/YOUR/DIR
```
It's also possible to chain multiple directories together. Note that order matters,
and the recipies will be copied in order of specification. So running something like
`mk test1 test2 test3 -- my_dir` does the following:
1) Copies test1's contents to `./my_dir`
2) Copies test2's contents to `./my_dir`
3) Copies test3's contents to `./my_dir`

The contents of the original `main.py` looked like this:
```python
# This file was placed in {{dir}} on {{day}}-{{month}}-{{year}}
def main() -> None:
    print("Welcome, {{user}}!")

if __name__ == '__main__':
    main()
```
So after copying it looks like this:
```python
# This file was placed in PATH/TO/YOUR/DIR on 20-12-2024
def main() -> None:
    print("Welcome, USER!")

if __name__ == '__main__':
    main()
```

Contributing
------------
While I doubt anyone ever will, pull requests are more than welcome.
This project has a special place in my heart as it is the longest
running project I have, and I will probably continue to update it as long as it
interests me and/or Steven.
