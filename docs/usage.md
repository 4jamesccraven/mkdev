# How to Use Mkdev
To get started, set up a directory as you normally would.
We'll use a simple python project as an example:
```
📂 example
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
Now the directory is saved and can be deployed anywhere by running `mk test`
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

Note how the special `{{substitution}}` syntax allows arbitrary text substitution in
the files. This behaviour can be configured. See the
[config wiki](https://github.com/4jamesccraven/mkdev/blob/main/docs/config.md) for more.
