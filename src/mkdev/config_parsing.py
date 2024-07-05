import os
import yaml
from dataclasses import dataclass
from typing import List, Tuple, Literal, Dict


Template = Dict[str, str | bool]
Recipe = List[str]


@dataclass
class Config:
    language: str
    extension: str
    templates: Dict[str, Template]
    recipes: Dict[str, Recipe]

    def build(self,
              recipe: str,
              directory: str,
              filename: str) -> None:
        '''
        Builds the recipe into the given file
        '''
        if not os.path.isdir(directory):
            os.makedirs(directory)

        steps = [self._parse_step(step) for step in self.recipes[recipe]]

        for step in steps:
            match step:
                # Make directories
                case ('dir', arg, False):
                    dir_path = os.path.join(directory, arg)

                    self._write_file(dir_path, 'directory')

                case ('dir', args, True):
                    dir_path = os.path.join(directory, *args)

                    self._write_file(dir_path, 'directory')

                # Make empty (i.e., placeholder) files
                case ('ph', arg, False):
                    ph_path = os.path.join(directory, arg)

                    self._write_file(ph_path, 'file')

                case ('ph', args, True):
                    ph_path = os.path.join(directory, *args)

                    self._write_file(ph_path, 'file')

                # Copy templates
                case ('tmp', arg, False):
                    template: Template = self.templates[arg]

                    filename = filename + self.extension\
                        if template['rename'] else template['filename']

                    tmp_path = os.path.join(directory, filename)

                    self._write_file(tmp_path,
                                     'file',
                                     contents=template['data'])

                case ('tmp', args, True):
                    template: Template = self.templates[args[-1]]

                    filename = filename + self.extension \
                        if template['rename'] else template['filename']

                    tmp_path = \
                        os.path.join(directory, *args[:-1], filename)

                    self._write_file(tmp_path,
                                     'file',
                                     contents=template['data'])

    @staticmethod
    def _write_file(path: str,
                    type: Literal['file', 'directory'],
                    contents: str = None) -> None:
        try:
            if type == 'directory':
                os.makedirs(path)

            else:
                with open(path, 'x', encoding='utf_8') as f:
                    if contents is not None:
                        f.write(contents)

        except FileExistsError:
            print(f'\'{path}\' already exists.')
        except PermissionError:
            print(f'mkdev: \'{path}\' access denied.')
        except Exception as e:
            print(f'Received unexpected error:\n {e}')

    @staticmethod
    def _parse_step(step: str) -> Tuple[str, str | List[str], bool]:
        if len(step.split(' ')) != 2:
            raise ValueError('Malformed build step:\nCommand should'
                             ' be of format [command argument], got '
                             f'\'{step}\'')

        command, argument = step.split(' ')

        if command not in ['dir', 'tmp', 'ph']:
            raise ValueError(f'{command} is not a valid command. '
                             ' Command must be one of dir, tmp, or ph.')

        multi = '|' in argument
        if multi:
            argument = argument.split('|')

        return (command, argument, multi)


def importLangs(confg_dir: str) -> 'List[Config]':
    '''
    Imports all configs from the config directory.

    Returns: a list of configs for each lang.

    Parameters:
    * confg_dir: the absolute path to the directory that stores configs
    '''
    files = os.listdir(confg_dir)
    files = [os.path.join(confg_dir, file) for file in files]

    configs = []
    for file in files:
        with open(file, 'r', encoding='utf_8') as f:
            data = yaml.safe_load(f)

            try:
                configs.append(Config(**data))
            except Exception as e:
                print(f'WARNING!:\nFailed to read config from {file}:\n{e}\n')

    return configs
