from textual.widget import Widget
from textual.app import ComposeResult
from textual.message import Message
from textual.reactive import Reactive
from textual.widgets import Rule, Input, Checkbox, TextArea, Button


class TemplateForm(Widget):
    name = Reactive('')
    filename = Reactive('')
    rename = Reactive(False)
    data = Reactive('')

    class Removed(Message):
        def __init__(self):
            super().__init__()

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
        self.post_message(self.Removed())
        self.set_timer(0.01, self.remove)
