from textual.app import ComposeResult
from textual.reactive import Reactive
from textual.widgets import Static, Rule, Input, TextArea, Button


class RecipeForm(Static):
    name = Reactive('')
    steps = Reactive('')

    def compose(self) -> ComposeResult:
        yield Rule()
        yield Button('X', variant='error')
        yield Input(placeholder='Recipe Name',
                    value=self.name)
        yield TextArea(text=self.steps)

    def on_button_pressed(self) -> None:
        self.remove()
