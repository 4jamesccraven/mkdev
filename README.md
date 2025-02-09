```
                                           _____  _____ 
                                          |____ ||  _  |
 _______ _     _ ______  _______ _    _       / /| |/' |
 |  |  | |____/  |     \ |______  \  /        \ \|  /| |
 |  |  | |    \_ |_____/ |______   \/     .___/ /\ |_/ /
                                          \____(_)\___/ 
```
![image](https://img.shields.io/badge/release-3.0.1-orange)
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

Installation
------------
WIP

Usage
-----
To get started, set up create a directory as you normally would.
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
mk test -- PATH/TO/YOUR/DIR
```
It's also possible to chain multiple directories together. Note that order matters,
and the recipies will be copied in order of specification. So running something like
`mk test1 test2 test3 -- my_dir` does the following:
1) Copies test1's contents to `./my_dir`
2) Copies test2's contents to `./my_dir`
3) Copies test3's contents to `./my_dir`

The contents of the original `main.py` looked like this:
```
# This file was placed in {{dir}} on {{day}}-{{month}}-{{year}}
def main() -> None:
    print("Welcome, {{user}}!")

if __name__ == '__main__':
    main()
```
So after copying it looks like this:
```
# This file was placed in PATH/TO/YOUR/DIR on 20-12-2024
def main() -> None:
    print("Welcome, USER!")

if __name__ == '__main__':
    main()
```
