from textual.widget import Widget
from textual.app import ComposeResult
from textual.reactive import Reactive
from textual.widgets import Rule, Input, Checkbox, TextArea, Button


class TemplateForm(Widget):
    name = Reactive('')
    filename = Reactive('')
    rename = Reactive(False)
    data = Reactive('')

    def compose(self) -> ComposeResult:
        yield Rule()
        yield Button('X', variant='error')
        yield Input(placeholder='Template Name',
                    value=self.name,
                    classes='name')
        yield Input(placeholder='Default File Name',
                    value=self.filename,
                    classes='file')
        yield Checkbox('Renameable', value=self.rename)
        yield TextArea(text=self.data)

    def on_button_pressed(self) -> None:
        self.remove()
