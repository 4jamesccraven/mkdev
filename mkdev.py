import os
import argparse
from shutil import copytree
import subprocess
from typing import List
import mkd_config as cfg
from argparse import Namespace
from platformdirs import user_config_dir

_NAME = 'mkdev'
_DESC = \
    'A command-line program that creates a develelopment environment' \
    ' from user-defined config files. ' \
    f' Note: User configs are in {user_config_dir(_NAME)}'

# For first run
if not os.path.isdir(user_config_dir(_NAME)):
    script = os.path.realpath(__file__)
    this_dir = os.path.dirname(script)
    def_cfg = os.path.join(this_dir, 'config')

    copytree(def_cfg, user_config_dir(_NAME))


def parse_args(cfgs: 'List[dict]') -> Namespace:
    langs = [cf['lang'] for cf in cfgs]

    PARSER = argparse.ArgumentParser(prog=_NAME,
                                     description=_DESC)

    SUBPS = PARSER.add_subparsers(title='Project', dest='lang', required=True)

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

    return PARSER.parse_args()


def main() -> None:
    # Get config directory
    config_path = user_config_dir(_NAME)

    # Parse the arguments using that path info
    configurations: 'List[dict]' = cfg.importLangs(config_path)
    args = parse_args(configurations)

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

    exit = cfg.build_recipe(recipe, config_path)

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
