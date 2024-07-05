def config_help(config_directory: str):
    help = 'Mkdev creates directory structures based off of yaml \n' \
           f'configuration files found in {config_directory}. you \n' \
           'can write these manually, or more simply generate them \n' \
           'using `mkdev edit`. \n' \
           '\n' \
           'The general structure of a configuration file is a language \n' \
           'name, an extension for the file type, template files that \n' \
           'will be written to files, and a list of instructions to \n' \
           'build a directory structure.'
    print(help)


def version(name: str, version: str):
    print(f'{name} version {version}\n\nThis project is made '
          'available under under the MIT License.\nSee '
          'https://github.com/4jamesccraven/mkdev for more '
          'information.')
