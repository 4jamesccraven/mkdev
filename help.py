def config_help(config_directory: str):
    help = f'''
Mkdev uses a config system to allow you to set up your directory structure
exactly as you like. It does this by creating a config directory, and
prefilling it with some sample configurations.

This is the structure:
{config_directory}
├── templates
│   ├── lang1
│   │   └── template for lang1
│   ├── lang2
│   │   ├── template for lang2
│   │   └── template for lang2
│   ├── lang3
│   └── ...
├── lang1.yaml
├── lang2.yaml
├── lang3.yaml
└── ...

Eseentiall, generally language setup is defined in its .yaml file, which give
the language a name (this will be the subcommand name, as well as the name of
its templates folder), associates an extension (like .cpp, or .js), template
names (which maps the in-configuration name to the associated template name)
and build recipes. Each of the latter two items should have at least one
sub-item called default. Build recipes make directories, templates or
placeholders. Tounderstand the behaviour further, just read the default c++
configuration. It will show you all features except for placeholders. To do a
placeholder in a build configuration, it is just the same as tmp, except the
command is ph and the argument is a filename literal.
'''
    print(help)


def version(name: str, version: str):
    print(f'{name} version {version}. This project is made '
          'available under under the MIT License. See '
          'https://github.com/4jamesccraven/mkdev for more '
          'information.')
