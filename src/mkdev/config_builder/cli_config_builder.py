import os
import yaml
from typing import Dict, Literal, Tuple

from textual.color import Color
from textual.app import App, ComposeResult, NoMatches
from textual.containers import VerticalScroll
from textual.widgets import Footer, Input, RichLog

from rich.syntax import Syntax

from . import _CONFIG
from .template_form import TemplateForm
from .recipe_form import RecipeForm
from .edit_dialogue import EditDialogue


class ConfigBuilder(App):
    # Key bindings and the functions they call
    BINDINGS = [
        ('q', 'quit', 'Quit'),
        ('t', 'add_template', 'Add template'),
        ('r', 'add_recipe', 'Add recipe'),
        ('ctrl+s', 'write_to_file', 'Save'),
        ('ctrl+o', 'open_file', 'Open existing')
    ]

    # Style
    CSS_PATH = 'config_builder.tcss'
    dark = True

    # Internal config data
    _current_working_config: Dict = {
        'language': '',
        'extension': '',
        'templates': {},
        'recipes': {},
    }
    # whether in the file dialogue or not
    dialogue = False

    # Whether editing, and what file is being edited
    editing = False
    file_being_edited = ''

    # Main Rendering
    def compose(self) -> ComposeResult:
        yield VerticalScroll(
            Input(placeholder='Language Name',
                  id='language'),
            Input(placeholder='Extension',
                  id='extension'),
            TemplateForm(),
            RecipeForm(),
            id='right-div',
        )
        yield VerticalScroll(
            RichLog(id='output', markup=True),
            RichLog(id='success', markup=True),
            id='left-div',
        )
        yield Footer()

    # Action definitions
    def action_add_template(self) -> 'TemplateForm':
        new_template = TemplateForm()
        # Try to mount before the first recipe if there
        # is one, otherwise mount 'wherever'
        try:
            first_recipe = (
                self
                .query_one('#right-div')
                .query(RecipeForm)
                .first()
            )
            (
                self
                .query_one('#right-div')
                .mount(new_template, before=first_recipe)
            )
        except NoMatches:
            (
                self
                .query_one('#right-div')
                .mount(new_template)
            )

        new_template.scroll_visible()
        return new_template

    def action_add_recipe(self) -> 'RecipeForm':
        new_recipe = RecipeForm()

        (
            self
            .query_one('#right-div')
            .mount(new_recipe)
        )

        new_recipe.scroll_visible()
        return new_recipe

    def action_write_to_file(self) -> None:
        self.write_to_file()

    def action_open_file(self) -> None:
        if not self.dialogue:
            self.push_screen(EditDialogue(),
                             callback=self.load_file)
            self.dialogue = True
        else:
            self.pop_screen()
            self.dialogue = False

    # Event listeners
    def on_input_changed(self) -> None:
        self.collect_data()

    def on_text_area_changed(self) -> None:
        self.collect_data()

    def on_checkbox_changed(self) -> None:
        self.collect_data()

    def on_template_form_removed(self) -> None:
        self.set_timer(0.01, self.collect_data)

    def on_recipe_form_removed(self) -> None:
        self.set_timer(0.01, self.collect_data)

    def on_mount(self) -> None:
        self.query_one(Footer).ctrl_to_caret = False

        curr_data = self._current_working_config
        curr_data = yaml.safe_dump(curr_data, sort_keys=False)
        curr_data = Syntax(curr_data, 'yaml')

        (
            self
            .query_one('#output')
            .clear()
            .write(curr_data)
        )

    # Main application logic
    def collect_data(self) -> None:
        '''
        Collects data from all user fields and renders it
        to preview before storing the data internally
        '''
        data = {
            'language': '',
            'extension': '',
            'templates': {},
            'recipes': {},
        }

        data['language'] = self.query_one('#language').value
        data['extension'] = self.query_one('#extension').value

        main_div = self.query_one('#right-div')

        for template in main_div.query('TemplateForm'):
            template.name = template.query_one('.name').value
            template.filename = template.query_one('.file').value
            template.rename = template.query_one('Checkbox').value
            template.data = template.query_one('TextArea').text

            data['templates'][template.name] = {
                'filename': template.filename,
                'rename': template.rename,
                'data': template.data
            }

        for recipe in main_div.query('RecipeForm'):
            recipe.name = recipe.query_one('Input').value
            recipe.steps = recipe.query_one('TextArea').text

            data['recipes'][recipe.name] = recipe.steps.split('\n')

        self._current_working_config = data

        curr_data = self._current_working_config
        curr_data = yaml.safe_dump(curr_data, sort_keys=False)
        curr_data = Syntax(curr_data, 'yaml')

        (
            self
            .query_one('#output')
            .clear()
            .write(curr_data)
        )

    def write_to_file(self) -> None:
        filename = self._current_working_config['language']
        filename = filename + '.yaml' if filename != '' \
            else filename

        path = os.path.join(_CONFIG, filename)

        try:
            if filename == '':
                raise ValueError('Filename cannot be empty.')

            # Only allowed to write over existing file if it was opened
            # via the open dialogue
            can_overwrite = self.editing and filename == self.file_being_edited
            mode = 'w' if can_overwrite else 'x'

            with open(path, mode, encoding='utf_8') as f:
                f.write(yaml.safe_dump(self._current_working_config))

                self.console_log(f'{filename} saved successfully.',
                                 status='ok')

        except Exception as e:
            self.console_log(str(e), status='bad')

    def load_file(self, filename: str) -> None:
        self.dialogue = False
        self.editing = True
        self.file_being_edited = filename[filename.rindex('/') + 1:]
        self.console_log(f'Opening {filename}...',
                         status='info')

        with open(filename, 'r', encoding='utf_8') as f:
            file_data = yaml.safe_load(f)

        # Ensure that data is properly formatted before proceeding
        valid_file = False
        match file_data:
            case {'language': str(_),
                  'extension': str(_),
                  'templates': dict(_),
                  'recipes': dict(_)}:
                valid_file = True

        if not valid_file:
            self.console_log(f'{filename} is not a valid file.',
                             status='bad')
            return

        # Change language and extension, remove all templates and recipes
        self.query_one('#language').value = file_data['language']
        self.query_one('#extension').value = file_data['extension']
        self.query('TemplateForm').remove()
        self.query('RecipeForm').remove()

        # Add a new template and recipe for each in the read-in data
        for _ in file_data['templates']:
            self.query_one('#right-div').mount(TemplateForm())
        for _ in file_data['recipes']:
            self.query_one('#right-div').mount(RecipeForm())

        # fill each template, and below each recipe, with the data
        # from the fine IN ORDER
        for form, template in zip(self.query(TemplateForm),
                                  file_data['templates']):
            form.name = template
            form.filename = file_data['templates'][template]['filename']
            form.rename = file_data['templates'][template]['rename']
            form.data = file_data['templates'][template]['data']

        for form, recipe in zip(self.query(RecipeForm),
                                file_data['recipes']):
            form.name = recipe
            form.steps = '\n'.join(file_data['recipes'][recipe])

    def console_log(self, text: str,
                    status: Literal['ok', 'bad', 'info']) -> None:
        '''
        Like console.log, but not that 👍
        '''
        valid_statuses = ['ok', 'bad', 'info']

        if status not in valid_statuses:
            raise ValueError('Invalid value for \'status\'. '
                             f' Must be one of {valid_statuses}')

        hatch_style: Tuple[str, Color] = None
        match status:
            case 'ok':
                hatch_style = ('right', Color(0, 128, 0, 0.2))
            case 'bad':
                hatch_style = ('cross', Color(255, 0, 0, 0.2))
            case 'info':
                hatch_style = ('>', Color(252, 148, 0, 0.2))
            case _:
                # Redundant, but 'added just in case'
                hatch_style = ('>', Color(252, 148, 0, 0.2))

        self.query_one('#success') \
            .write(text) \
            .styles.hatch = hatch_style


__all__ = ['ConfigBuilder']
