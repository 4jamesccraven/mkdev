# IMPORTS

import os
import yaml
from typing import Dict

from platformdirs import user_config_dir

from textual.color import Color
from textual.app import App, ComposeResult
from textual.containers import VerticalScroll
from textual.widgets import Footer, Input, RichLog

from config_builder.cli.template_form import TemplateForm
from config_builder.cli.recipe_form import RecipeForm

_NAME = 'mkdev'


class ConfigBuilder(App):
    BINDINGS = [
        ('q', 'quit', 'Quit'),
        ('t', 'add_template', 'Add template'),
        ('T', 'remove_template', 'Remove template'),
        ('r', 'add_recipe', 'Add recipe'),
        ('R', 'remove_recipe', 'Remove recipe'),
        ('ctrl+s', 'write_to_file', 'Save'),
    ]
    CSS_PATH = 'config_builder.tcss'
    dark = True
    _current_working_config: Dict = {
        'language': '',
        'extension': '',
        'templates': {},
        'recipes': {},
    }

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
        self.query_one('#right-div').mount(new_template)
        new_template.scroll_visible()
        return new_template

    def action_remove_template(self) -> None:
        templates = self.query('TemplateForm')
        if templates:
            templates.last().remove()
        self.collect_data()

    def action_add_recipe(self) -> 'RecipeForm':
        new_recipe = RecipeForm()
        self.query_one('#right-div').mount(new_recipe)
        new_recipe.scroll_visible()
        return new_recipe

    def action_remove_recipe(self) -> None:
        recipes = self.query('RecipeForm')
        if recipes:
            recipes.last().remove()
        self.collect_data()

    def action_write_to_file(self) -> None:
        self.write_to_file()

    # Event listeners
    def on_input_changed(self) -> None:
        self.collect_data()

    def on_text_area_changed(self) -> None:
        self.collect_data()

    def on_checkbox_changed(self) -> None:
        self.collect_data()

    def on_mount(self) -> None:
        self.query_one('#output').clear().write(
            yaml.safe_dump(self._current_working_config, sort_keys=False)
        )

    # Main application logic
    def collect_data(self) -> None:
        data = {
            'language': '',
            'extension': '',
            'templates': {},
            'recipes': {},
        }

        # presumed_write_path = \
        #     os.path.join(user_config_dir(_NAME),
        #                  self.query_one('#language').value + '.yaml')
        # self.console_log(presumed_write_path)

        # if os.path.isfile(presumed_write_path):
        #     data = self.set_editing(presumed_write_path)

        #     conditions = [val not in data.keys() for val in
        #                   ['language', 'extension', 'templates', 'recipes']]
        #     if any(conditions):
        #         assert not self.editing
        #         assert self.curr_document == ''
        #         data = self.set_editing(presumed_write_path)
        #     else:
        #         self.query('TemplateForm').remove()
        #         self.query('RecipeForm').remove()
        #         self.query_one('#extension').clear()
        #         self.query_one('#extension').value = data['extension']

        data['language'] = self.query_one('#language').value
        data['extension'] = self.query_one('#extension').value

        for template in self.query('TemplateForm'):
            data['templates'][template.query_one('.name').value] = {
                'filename': template.query_one('.file').value,
                'rename': template.query_one('Checkbox').value,
                'data': template.query_one('TextArea').text
            }

        for recipe in self.query('RecipeForm'):
            data['recipes'][recipe.query_one('Input').value] = \
                recipe.query_one('TextArea').text.split('\n')

        self._current_working_config = data

        self.query_one('#output').clear().write(
            yaml.safe_dump(self._current_working_config, sort_keys=False)
        ).refresh()

    def write_to_file(self) -> None:
        filename = self._current_working_config['language']
        filename = filename + '.yaml' if filename != '' else filename
        path = os.path.join(user_config_dir('mkdev'), filename)

        try:
            if filename == '':
                raise ValueError('Filename cannot be empty.')

            mode = 'w' if self.editing else 'x'
            with open(path, mode, encoding='utf_8') as f:
                f.write(yaml.safe_dump(self._current_working_config))

                self.query_one('#success') \
                    .write(f'{filename} saved successfully!') \
                    .styles.hatch = ('right', Color(0, 128, 0, 0.2))

        except Exception as e:
            self.query_one('#success') \
                .write(str(e)) \
                .styles.hatch = ('cross', Color(255, 0, 0, 0.2))

    def console_log(self, text: str) -> None:
        self.query_one('#success') \
            .write(text) \
            .styles.hatch = ('horizontal', Color(252, 148, 0, 0.2))


__all__ = ['ConfigBuilder']
