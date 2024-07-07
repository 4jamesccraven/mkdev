import os
import subprocess
from shutil import copytree
from typing import List, Tuple
from platformdirs import user_config_dir
from .mkdev_help import config_help, version
from .config_builder.cli_config_builder import ConfigBuilder
from .config_parsing import Config, importLangs
from argparse import Namespace, ArgumentParser

_NAME = 'mkdev'
_VERS = '2.0'
_CONFIG = user_config_dir(_NAME)
_DESC = \
    'A command-line program that creates a development environment' \
    ' from user-defined configuration files.' \
    f'\nNote: User configs are stored at {_CONFIG}'


def init_config(force: bool) -> None:
    '''
    Initialises mkdev for use by copying the default configurations
    for the application to the configuration directory. Overwrites
    existing configs if force is true

    Example: mkdev init --force
    '''
    # Checks to see if the directory exists
    config_exists = os.path.isdir(_CONFIG)

    # If it does, get a list of the files
    if config_exists:
        files_in_dir = [os.path.join(_CONFIG, file)
                        for file in os.listdir(_CONFIG)]
    conditions = [
        not config_exists,
        len(files_in_dir) == 0 if config_exists else False,
        force,
    ]

    if any(conditions):
        if force and config_exists:
            for file in files_in_dir:
                os.remove(file)
            os.rmdir(_CONFIG)

        script = os.path.realpath(__file__)
        this_dir = os.path.dirname(script)
        def_cfg = os.path.join(this_dir, 'config')

        copytree(def_cfg, _CONFIG, dirs_exist_ok=True)

        # Changes default permissions, mainly added
        # because in nix these files are read only
        # when copied
        os.chmod(_CONFIG, 0o755)
        for file in os.listdir(_CONFIG):
            path = os.path.join(_CONFIG, file)

            os.chmod(path, 0o764)


def parse_args(cfgs: 'List[Config | None]'
               ) -> Tuple[Namespace, ArgumentParser]:
    '''
    Parses the command line arguments.
    '''
    langs = [cf.language for cf in cfgs]

    PARSER = ArgumentParser(prog=_NAME,
                            description=_DESC)
    PARSER.add_argument('--config-help',
                        help='Displays information on configuring mkdev.',
                        action='store_true')
    PARSER.add_argument('--version',
                        help='See version info.',
                        action='store_true')

    SUBPS = PARSER.add_subparsers(title='Language/Action', dest='action')

    SUBPS.add_parser('edit',
                     help='Edit configurations in GUI.')
    SUBPS.add_parser('init',
                     help='Sets up configuration files for use.') \
        .add_argument('--force',
                      help=f'Forces {_NAME} to re-'
                      'write configs.',
                      action='store_true')

    S_PARSERS = {}
    for lang in langs:
        CFG_DATA = next(filter(lambda cfg: cfg.language == lang, cfgs))

        S_PARSERS[lang] = SUBPS.add_parser(lang)
        S_PARSERS[lang].add_argument('directory',
                                     help='Directory to build'
                                     ' (Default \'.\')',
                                     nargs='?',
                                     default=os.getcwd())
        S_PARSERS[lang].add_argument('file',
                                     help='Name to assign to'
                                     ' to the default template'
                                     ' (default \'main\')',
                                     nargs='?',
                                     default='main')
        S_PARSERS[lang].add_argument('-c', '--code',
                                     help='Opens Visual Studio '
                                     'Code on exit.',
                                     action='store_true')
        S_PARSERS[lang].add_argument('-r', '--recipe',
                                     help='Build recipe to use '
                                     ' (Default \'default\').',
                                     default='default',
                                     choices=CFG_DATA.recipes.keys())
        S_PARSERS[lang].add_argument('-v', '--verbose',
                                     help='Prints debug info.',
                                     action='store_true')

    return PARSER.parse_args(), PARSER


def main() -> None:
    # Parse the arguments using that path info
    try:
        configurations: 'List[Config | None]' = importLangs(_CONFIG)
    except FileNotFoundError:
        configurations = []

    args, PARSER = parse_args(configurations)

    # Handle non-language options, i.e., auxillary functions
    if args.config_help:
        config_help(_CONFIG)
        return
    if args.version:
        version(_NAME, _VERS)
        return
    if args.action == 'init':
        init_config(args.force)
        return
    if args.action == 'edit':
        editor = ConfigBuilder()
        editor.run()
        return
    elif not args.action:
        PARSER.print_usage()
        print('mkdev: error: the following arguments are required: action')
        return

    # Filter the correct language data from the list of data
    build: Config = next(filter(lambda cfg: cfg.language == args.action,
                                configurations))

    if args.verbose:
        print(f'{build=}')

    build.build(recipe=args.recipe,
                directory=args.directory,
                filename=args.file)

    if args.code:
        try:
            completed_process = subprocess.Popen(['code', args.directory])
            completed_process.wait()
        except subprocess.CalledProcessError as e:
            print(f'Error launching VSCode\n{e.output}')


if __name__ == '__main__':
    main()
