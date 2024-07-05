import os


def config_help(config_directory: str):
    script = os.path.realpath(__file__)
    this_dir = os.path.dirname(script)
    def_help = os.path.join(this_dir, 'help.txt')

    with open(def_help, 'r', encoding='utf_8') as f:
        for line in f:
            print(line.strip().replace('${CONFIG}', config_directory))
    print(help)


def version(name: str, version: str):
    print(f'{name} version {version}\n\nThis project is made '
          'available under under the MIT License.\nSee '
          'https://github.com/4jamesccraven/mkdev for more '
          'information.')
