from platformdirs import user_config_dir

_CONFIG = user_config_dir('mkdev')

__all__ = [
    '.cli_config_builder.ConfigBuilder',
    '.cli_config_builder',
    '.recipe_form',
    '.template_form'
]
