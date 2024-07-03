import os
import yaml
from typing import Dict, Literal, Tuple

from textual.color import Color
from textual.app import App, ComposeResult, NoMatches
from textual.containers import VerticalScroll
from textual.widgets import Footer, Input, RichLog

from rich.syntax import Syntax

from config_builder import _CONFIG
from config_builder.template_form import TemplateForm
from config_builder.recipe_form import RecipeForm
from config_builder.edit_dialogue import EditDialogue


class ConfigBuilder(App):
    BINDINGS = [
        ('q', 'quit', 'Quit'),
        ('t', 'add_template', 'Add template'),
        ('r', 'add_recipe', 'Add recipe'),
        ('ctrl+s', 'write_to_file', 'Save'),
        ('ctrl+o', 'open_file', 'Open existing')
    ]
    CSS_PATH = 'config_builder.tcss'
    dark = True
    _current_working_config: Dict = {
        'language': '',
        'extension': '',
        'templates': {},
        'recipes': {},
    }
    dialogue = False
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
        try:
            first_recipe = self.query_one('#right-div') \
                               .query(RecipeForm) \
                               .first()
            self.query_one('#right-div') \
                .mount(new_template, before=first_recipe)
        except NoMatches:
            self.query_one('#right-div') \
                .mount(new_template)
        new_template.scroll_visible()
        return new_template

    def action_remove_template(self) -> None:
        templates = self.query('TemplateForm')
        if templates:
            templates.last().remove()
        self.collect_data()

    def action_add_recipe(self) -> 'RecipeForm':
        new_recipe = RecipeForm()
        self.query_one('#right-div') \
            .mount(new_recipe)
        new_recipe.scroll_visible()
        return new_recipe

    def action_remove_recipe(self) -> None:
        recipes = self.query('RecipeForm')
        if recipes:
            recipes.last().remove()
        self.collect_data()

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

    def on_mount(self) -> None:
        self.query_one('#output').clear().write(
            Syntax(yaml.safe_dump(self._current_working_config,
                                  sort_keys=False),
                   "yaml")
        )

    # Main application logic
    def collect_data(self) -> None:
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

        self.query_one('#output').clear().write(
            Syntax(yaml.safe_dump(self._current_working_config,
                                  sort_keys=False),
                   "yaml")
        ).refresh()

    def write_to_file(self) -> None:
        filename = self._current_working_config['language']
        filename = filename + '.yaml' if filename != '' else filename
        path = os.path.join(_CONFIG, filename)

        try:
            if filename == '':
                raise ValueError('Filename cannot be empty.')

            can_overwrite = self.editing and filename == self.file_being_edited
            mode = 'w' if can_overwrite else 'x'
            with open(path, mode, encoding='utf_8') as f:
                f.write(yaml.safe_dump(self._current_working_config))

                self.console_log(f'{filename} saved successfully!',
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

        self.query_one('#language').value = file_data['language']
        self.query_one('#extension').value = file_data['extension']
        self.query('TemplateForm').remove()
        self.query('RecipeForm').remove()

        for _ in file_data['templates']:
            self.query_one('#right-div').mount(TemplateForm())
        for _ in file_data['recipes']:
            self.query_one('#right-div').mount(RecipeForm())

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
                hatch_style = ('>', Color(252, 148, 0, 0.2))

        self.query_one('#success') \
            .write(text) \
            .styles.hatch = hatch_style


__all__ = ['ConfigBuilder']
