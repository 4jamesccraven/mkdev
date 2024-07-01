import os
import subprocess
from shutil import copytree
from config_parsing import Config, importLangs
from typing import List, Tuple
from help import config_help, version
from platformdirs import user_config_dir
from argparse import Namespace, ArgumentParser

_NAME = 'mkdev'
_VERS = 'unstable'
_CONFIG = user_config_dir(_NAME)
_DESC = \
    'A command-line program that creates a development environment' \
    ' from user-defined configuration files. ' \
    f' Note: User configs are stored at {_CONFIG}'

# For first run, copy default configs if a directory doesn't exist
# or if it is empty
if not os.path.isdir(_CONFIG) or len(os.listdir(_CONFIG)) == 0:
    script = os.path.realpath(__file__)
    this_dir = os.path.dirname(script)
    def_cfg = os.path.join(this_dir, 'config')

    try:
        copytree(def_cfg, _CONFIG, dirs_exist_ok=True)
    except Exception as e:
        print(f'Warning: error writing default configurations '
              f'please ensure that {_CONFIG} does not already exist. '
              f'Further info:\n{e}')


def parse_args(cfgs: 'List[Config]') -> Tuple[Namespace, ArgumentParser]:
    langs = [cf.language for cf in cfgs]

    PARSER = ArgumentParser(prog=_NAME,
                            description=_DESC)
    PARSER.add_argument('--config-help',
                        help='Displays information on configuring mkdev.',
                        action='store_true')
    PARSER.add_argument('--version',
                        help='See version info.',
                        action='store_true')

    SUBPS = PARSER.add_subparsers(title='Language', dest='lang')

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
    configurations: 'List[Config]' = importLangs(_CONFIG)

    args, PARSER = parse_args(configurations)

    if args.config_help:
        config_help(_CONFIG)
        return
    if args.version:
        version(_NAME, _VERS)
        return
    elif not args.lang:
        PARSER.print_usage()
        print('mkdev: error: the following arguments are required: lang')
        return

    # Filter the correct language data from the list of data
    build: Config = next(filter(lambda cfg: cfg.language == args.lang,
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
