import os
import subprocess
from shutil import copytree
import config_parsing as cfg
from typing import List, Tuple
from help import config_help, version
from platformdirs import user_config_dir
from argparse import Namespace, ArgumentParser

_NAME = 'mkdev'
_VERS = '1.2'
_CONFIG = user_config_dir(_NAME)
_DESC = \
    'A command-line program that creates a develelopment environment' \
    ' from user-defined config files. ' \
    f' Note: User configs are in {_CONFIG}'

# For first run, copy default configs if a directory doesn't exist
# or if it is empty
if not os.path.isdir(_CONFIG) or len(os.listdir(_CONFIG)) == 0:
    script = os.path.realpath(__file__)
    this_dir = os.path.dirname(script)
    def_cfg = os.path.join(this_dir, 'config')

    copytree(def_cfg, _CONFIG)


def parse_args(cfgs: 'List[dict]') -> Tuple[Namespace, ArgumentParser]:
    langs = [cf['lang'] for cf in cfgs]

    PARSER = ArgumentParser(prog=_NAME,
                            description=_DESC)
    PARSER.add_argument('--config-help',
                        help='Displays information on configuring mkdev.',
                        action='store_true')
    PARSER.add_argument('--version',
                        help='See version info.',
                        action='store_true')

    SUBPS = PARSER.add_subparsers(title='Project', dest='lang')

    S_PARSERS = {}
    for lang in langs:
        CFG_DATA = next(filter(lambda cfg: cfg['lang']
                        == lang, cfgs))

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
                                     help='Opens VSCode on exit.',
                                     action='store_true')
        S_PARSERS[lang].add_argument('-r', '--recipe',
                                     help='Build recipe to use '
                                     ' (Default \'default\').',
                                     default='default',
                                     choices=CFG_DATA['build'].keys())
        S_PARSERS[lang].add_argument('-v', '--verbose',
                                     help='Prints debug info.',
                                     action='store_true')

    return PARSER.parse_args(), PARSER


def main() -> None:
    # Parse the arguments using that path info
    configurations: 'List[dict]' = cfg.importLangs(_CONFIG)

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
    lang_data = next(filter(lambda cfg: cfg['lang'] == args.lang,
                            configurations))

    if not os.path.isdir(args.directory):
        os.makedirs(args.directory)

    # Instantiate a recipe
    recipe = cfg.Recipe(
        lang=args.lang,
        name=args.recipe,
        data=lang_data,
        build_dir=args.directory,
        build_file=args.file,
    )

    verbose = args.verbose
    code = args.code

    if verbose:
        print(args)
        print()
        print(recipe)

    cfg.build_recipe(recipe, _CONFIG)

    if exit:
        return

    if code:
        try:
            subprocess.run(['code', args.directory])
        except Exception as e:
            print(f'Unable to launch code:\n{e}')
            print('Try launching manually')


if __name__ == '__main__':
    main()
