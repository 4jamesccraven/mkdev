import os
import yaml
from typing import Dict
from platformdirs import user_config_dir
from textual.app import App, ComposeResult
from textual.containers import Container, VerticalScroll
from textual.widgets import Footer, \
                            Input, \
                            Static, \
                            Checkbox, \
                            TextArea, \
                            Log, \
                            Rule


class ConfigBuilder(App):
    BINDINGS = [
        ('q', 'quit', 'Quit'),
        ('t', 'add_template', 'Add template'),
        ('T', 'remove_template', 'Remove template'),
        ('r', 'add_recipe', 'Add recipe'),
        ('R', 'remove_recipe', 'Remove recipe'),
        ('ctrl+o', 'write_to_file', 'Save'),
    ]
    CSS_PATH = 'config_builder.tcss'
    dark = True

    _current_working_config: Dict = {'language': ''}

    def compose(self) -> ComposeResult:
        yield VerticalScroll(
            Input(placeholder='Language Name',
                  id='language'),
            Input(placeholder='Extension',
                  id='extension'),
            id='right-div',
        )
        yield Container(
            Log(),
            id='left-div',
        )
        yield Footer()

    def action_add_template(self) -> None:
        new_template = TemplateForm()
        self.query_one('#right-div').mount(new_template)
        new_template.scroll_visible()

    def action_remove_template(self) -> None:
        templates = self.query('TemplateForm')
        if templates:
            templates.last().remove()

    def action_add_recipe(self) -> None:
        new_recipe = RecipeForm()
        self.query_one('#right-div').mount(new_recipe)
        new_recipe.scroll_visible()

    def action_remove_recipe(self) -> None:
        recipes = self.query('RecipeForm')
        if recipes:
            recipes.last().remove()

    def on_input_changed(self) -> None:
        self.collect_data()

    def on_text_area_changed(self) -> None:
        self.collect_data()

    def on_checkbox_changed(self) -> None:
        self.collect_data()

    def collect_data(self) -> None:
        data = {
            'language': self.query_one('#language').value,
            'extension': self.query_one('#extension').value,
            'templates': {},
            'recipes': {},
        }

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

        self.query_one('Log').clear().write_line(
            yaml.safe_dump(self._current_working_config, sort_keys=False)
        )

    def action_write_to_file(self) -> None:
        filename = self._current_working_config['language'] + '.yaml'
        path = os.path.join(user_config_dir('mkdev'), filename)

        try:
            with open(path, 'x', encoding='utf_8') as f:
                f.write(yaml.safe_dump(self._current_working_config))
                self.query_one('Footer').add_class('succeeded')
        except FileExistsError:
            self.query_one('Footer').add_class('failed')

    def on_mount(self) -> None:
        self.query_one('Log').clear().write_line(
            yaml.safe_dump(self._current_working_config, sort_keys=False)
        )


class TemplateForm(Static):
    def compose(self) -> ComposeResult:
        yield Rule()
        yield Input(placeholder='Template Name', classes='name')
        yield Input(placeholder='Default File Name', classes='file')
        yield Checkbox('Renameable')
        yield TextArea()


class RecipeForm(Static):
    def compose(self) -> ComposeResult:
        yield Rule()
        yield Input(placeholder='Recipe Name')
        yield TextArea()


if __name__ == '__main__':
    app = ConfigBuilder()
    app.run()
