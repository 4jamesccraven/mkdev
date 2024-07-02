from typing import List
from textual.app import ComposeResult
from textual.widgets import Static, Rule, Input, TextArea


class RecipeForm(Static):
    def compose(self) -> ComposeResult:
        yield Rule()
        yield Input(placeholder='Recipe Name')
        yield TextArea()

    def update(self, name: str, text: List[str]) -> 'RecipeForm':
        self.query_one('Input').value = name
        self.query_one('TextArea').text = '\n'.join(text)
        return self
