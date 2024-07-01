def config_help(config_directory: str):
    help = '''
    Mkdev uses a config system to allow you to set up your directory structure
    exactly as you like. It does this by creating a config directory, and
    prefilling it with some sample configurations.

    This is the truncated structure of a config (c++ as an example):
    language: c++
    extension: .cpp
    templates:      # the text that will be written, and other info
      default:
        ...
    recipes:
      default:
        ...
    '''
    print(help)


def version(name: str, version: str):
    print(f'{name} version {version}. This project is made '
          'available under under the MIT License. See '
          'https://github.com/4jamesccraven/mkdev for more '
          'information.')
