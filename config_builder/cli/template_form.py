from typing import Dict
from textual.app import ComposeResult
from textual.widgets import Static, Rule, Input, Checkbox, TextArea


class TemplateForm(Static):
    def compose(self) -> ComposeResult:
        yield Rule()
        yield Input(placeholder='Template Name', classes='name')
        yield Input(placeholder='Default File Name', classes='file')
        yield Checkbox('Renameable')
        yield TextArea()

    def update(self, name: str, new_data: Dict) -> 'TemplateForm':
        self.query_one('.name').value = name
        self.query_one('.file').value = new_data['filename']
        self.query_one('Checkbox').value = new_data['rename']
        self.query_one('TextArea').text = new_data['data']
        return self
