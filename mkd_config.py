import os
import yaml
from typing import List, Tuple
from dataclasses import dataclass


@dataclass
class Recipe:
    lang: str
    name: str
    data: dict
    build_dir: str
    build_file: str


def importLangs(confg_dir: str) -> 'List[dict]':
    '''
    Imports all configs from the config directory.

    Returns: a list of configs for each lang.

    Parameters:
    * confg_dir: the absolute path to the directory that stores configs
    '''
    files = os.listdir(confg_dir)
    files = [os.path.join(confg_dir, file) for file in files
             if file != 'templates']

    cfgs = []
    for file in files:
        with open(file, 'r') as f:
            data = yaml.load(f, Loader=yaml.SafeLoader)

            cfgs.append(data)

    # ugly way of making sure that user configs are
    # mostly right...
    r_val = []
    for cf in cfgs:
        match cf:
            case {'lang': _,
                  'ext': _,
                  'templates': templates,
                  'build': builds} \
                  if isinstance(templates, dict) and isinstance(builds, dict):
                r_val.append(cf)

    return r_val


def parse_step(step: str) -> Tuple[str, str | List[str], bool]:
    '''
    Parses a step from a build recipe for easier use.
    '''

    if len(step.split(' ')) != 2:
        raise ValueError(f'Malformed build step:\nCommand should'
                         f' be of format [command argument], got [{step}]')

    command, argument = step.split(' ')

    if command not in ['dir', 'tmp', 'ph']:
        raise ValueError('Error: Steps should contain a valid command'
                         f' and a single argument. Got \'{step}\'')

    multi = '|' in argument
    if multi:
        argument = argument.split('|')

    return (command, argument, multi)


def import_template(recipe: Recipe,
                    tmp_dir: str,
                    tmp_name: str) -> Tuple['List[str]', bool]:
    template_rl = os.path.join(tmp_dir, recipe.data['templates'][tmp_name])

    rename = '[r]' in recipe.data['templates'][tmp_name]

    with open(template_rl, 'r') as f:
        template = [line for line in f]

    return template, rename


def build_recipe(recipe: Recipe, abs_cfg: str) -> bool:
    '''
    Builds directory of project from a recipe
    '''
    template_dir = os.path.join(abs_cfg, 'templates', recipe.lang)

    if not os.path.isdir(recipe.build_dir):
        os.makedirs(recipe.build_dir)

    steps = [parse_step(step)
             for step in recipe.data['build'][recipe.name]]

    for step in steps:
        match step:
            case ('dir', arg, False):
                dir_path = os.path.join(recipe.build_dir, arg)
                try:
                    os.mkdir(dir_path)
                except FileExistsError:
                    print(f'{dir_path} already exists.')

            case ('dir', args, True):
                dir_path = os.path.join(recipe.build_dir, *args)
                try:
                    os.makedirs(recipe.build_dir, *arg)
                except FileExistsError:
                    print('One or more of the directories in path'
                          f' {dir_path} exist(s) already.')
                    return True

            case ('ph', arg, False):
                ph_path = os.path.join(recipe.build_dir, arg)
                try:
                    with open(ph_path, 'x'):
                        ...
                except FileExistsError:
                    print(f'{ph_path} already exists.')

            case ('ph', args, True):
                ph_path = os.path.join(recipe.build_dir, *args)
                try:
                    with open(ph_path, 'x'):
                        ...
                except FileExistsError:
                    print(f'{ph_path} already exists.')
                except FileNotFoundError as e:
                    print(f'{ph_path} not found:\n{e}')

            case ('tmp', arg, False):
                tmp, rename = import_template(recipe, template_dir, arg)
                filename = recipe.build_file + recipe.data['ext'] \
                    if rename else arg

                write_path = os.path.join(recipe.build_dir, filename)
                with open(write_path, 'w') as f:
                    f.writelines(tmp)

            case ('tmp', args, True):
                tmp, rename = import_template(recipe, template_dir, args[-1])
                filename = recipe.build_file + recipe.data['ext'] \
                    if rename else args[-1]

                write_path = \
                    os.path.join(recipe.build_dir, *args[:-1], filename)
                with open(write_path, 'w') as f:
                    f.writelines(tmp)

    return False
