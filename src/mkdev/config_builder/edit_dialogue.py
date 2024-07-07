from . import _CONFIG
from textual.app import ComposeResult
from textual.screen import Screen
from textual.widgets import DirectoryTree, Label


class EditDialogue(Screen):
    def compose(self) -> ComposeResult:
        yield Label('Select configuration to edit:')
        yield DirectoryTree(_CONFIG)

    def on_tree_node_highlighted(self,
                                 event: DirectoryTree.NodeHighlighted) -> None:
        '''
        Dismisses the screen and returns the selected file as a
        string which is passed to the callback function
        '''
        self.dismiss(str(event.node.data.path))
